package main

import (
	"context"
	stdjson "encoding/json"
	"errors"
	"flag"
	"fmt"
	"log"
	"net/http"
	"os"
	"path/filepath"
	"sync"
	"time"

	box "github.com/sagernet/sing-box"
	"github.com/sagernet/sing-box/adapter/endpoint"
	"github.com/sagernet/sing-box/adapter/inbound"
	"github.com/sagernet/sing-box/adapter/outbound"
	boxservice "github.com/sagernet/sing-box/adapter/service"
	"github.com/sagernet/sing-box/dns"
	"github.com/sagernet/sing-box/dns/transport"
	"github.com/sagernet/sing-box/dns/transport/local"
	"github.com/sagernet/sing-box/option"
	"github.com/sagernet/sing-box/protocol/direct"
	"github.com/sagernet/sing-box/protocol/mixed"
	"github.com/sagernet/sing-box/protocol/socks"
	"github.com/sagernet/sing-box/protocol/tun"
	singjson "github.com/sagernet/sing/common/json"
	"golang.org/x/sys/windows"
	"golang.org/x/sys/windows/svc"
	"golang.org/x/sys/windows/svc/mgr"
)

const (
	serviceName = "LibreRTCNetService"
	displayName = "LibreRTC Net Service"
	listenAddr  = "127.0.0.1:38741"
)

func main() {
	install := flag.Bool("install", false, "install Windows service")
	uninstall := flag.Bool("uninstall", false, "uninstall Windows service")
	console := flag.Bool("console", false, "run as a console process")
	flag.Parse()

	if *install {
		must(installService())
		return
	}
	if *uninstall {
		must(uninstallService())
		return
	}

	interactive, err := svc.IsAnInteractiveSession()
	must(err)
	if interactive || *console {
		must(run(context.Background()))
		return
	}
	must(svc.Run(serviceName, service{}))
}

type service struct{}

func (service) Execute(args []string, requests <-chan svc.ChangeRequest, changes chan<- svc.Status) (bool, uint32) {
	ctx, cancel := context.WithCancel(context.Background())
	done := make(chan error, 1)

	changes <- svc.Status{State: svc.StartPending}
	go func() { done <- run(ctx) }()
	changes <- svc.Status{State: svc.Running, Accepts: svc.AcceptStop | svc.AcceptShutdown}

	for {
		select {
		case request := <-requests:
			switch request.Cmd {
			case svc.Stop, svc.Shutdown:
				changes <- svc.Status{State: svc.StopPending}
				cancel()
				<-done
				return false, 0
			case svc.Interrogate:
				changes <- request.CurrentStatus
			}
		case err := <-done:
			if err != nil && !errors.Is(err, http.ErrServerClosed) {
				return false, 1
			}
			return false, 0
		}
	}
}

type Engine interface {
	Start(context.Context, StartRequest) error
	Stop(context.Context) error
	Status() EngineStatus
}

type EngineStatus struct {
	Running   bool   `json:"running"`
	Mode      string `json:"mode"`
	Socks     string `json:"socks"`
	Proxy     string `json:"proxy"`
	UpdatedAt string `json:"updated_at"`
	Message   string `json:"message"`
}

type StartRequest struct {
	Mode       string `json:"mode"`
	SocksHost string `json:"socks_host"`
	SocksPort int    `json:"socks_port"`
	ListenHost string `json:"listen_host"`
	ListenPort int    `json:"listen_port"`
	DNS       string `json:"dns"`
}

type singBoxEngine struct {
	mu       sync.Mutex
	instance *box.Box
	cancel   context.CancelFunc
	status   EngineStatus
}

func (engine *singBoxEngine) Start(ctx context.Context, request StartRequest) error {
	engine.mu.Lock()
	defer engine.mu.Unlock()
	if err := engine.stopLocked(); err != nil {
		return err
	}
	options, err := buildSingBoxOptions(request)
	if err != nil {
		return err
	}
	boxCtx, cancel := context.WithCancel(singBoxContext(context.Background()))
	instance, err := box.New(box.Options{Context: boxCtx, Options: options})
	if err != nil {
		cancel()
		return fmt.Errorf("create sing-box engine: %w", err)
	}
	if err := instance.Start(); err != nil {
		cancel()
		_ = instance.Close()
		return fmt.Errorf("start sing-box engine: %w", err)
	}
	select {
	case <-ctx.Done():
		cancel()
		_ = instance.Close()
		return ctx.Err()
	default:
	}
	engine.instance = instance
	engine.cancel = cancel
	mode := request.Mode
	if mode == "" {
		mode = "tun"
	}
	proxy := ""
	if mode == "proxy" {
		proxy = fmt.Sprintf("%s:%d", request.ListenHost, request.ListenPort)
	}
	engine.status = EngineStatus{
		Running:   true,
		Mode:      mode,
		Socks:     fmt.Sprintf("%s:%d", request.SocksHost, request.SocksPort),
		Proxy:     proxy,
		UpdatedAt: time.Now().Format(time.RFC3339),
		Message:   "sing-box embedded engine running",
	}
	return nil
}

func (engine *singBoxEngine) Stop(_ context.Context) error {
	engine.mu.Lock()
	defer engine.mu.Unlock()
	if err := engine.stopLocked(); err != nil {
		return err
	}
	engine.status.Running = false
	engine.status.UpdatedAt = time.Now().Format(time.RFC3339)
	engine.status.Message = "stopped"
	return nil
}

func (engine *singBoxEngine) Status() EngineStatus {
	engine.mu.Lock()
	defer engine.mu.Unlock()
	return engine.status
}

func (engine *singBoxEngine) stopLocked() error {
	var closeErr error
	if engine.cancel != nil {
		engine.cancel()
		engine.cancel = nil
	}
	if engine.instance != nil {
		closeErr = engine.instance.Close()
		engine.instance = nil
	}
	return closeErr
}

func buildSingBoxOptions(request StartRequest) (option.Options, error) {
	if request.Mode == "proxy" {
		return buildProxyOptions(request)
	}
	return buildTunOptions(request)
}

func buildTunOptions(request StartRequest) (option.Options, error) {
	if request.SocksHost == "" {
		request.SocksHost = "127.0.0.1"
	}
	if request.SocksPort <= 0 || request.SocksPort > 65535 {
		return option.Options{}, fmt.Errorf("invalid SOCKS port: %d", request.SocksPort)
	}
	dnsServer := dnsServerHost(request.DNS)
	raw := fmt.Sprintf(`{
  "log": {
    "level": "warn",
    "timestamp": true
  },
  "dns": {
    "servers": [
      {
        "type": "https",
        "tag": "remote-dns",
        "server": %q,
        "server_port": 443,
        "path": "/dns-query",
        "detour": "socks-out"
      }
    ],
    "final": "remote-dns",
    "strategy": "ipv4_only"
  },
  "inbounds": [
    {
      "type": "tun",
      "tag": "tun-in",
      "interface_name": "LibreRTC",
      "address": ["172.19.0.1/30"],
      "mtu": 1500,
      "auto_route": true,
      "strict_route": true,
      "stack": "mixed"
    }
  ],
  "outbounds": [
    {
      "type": "socks",
      "tag": "socks-out",
      "server": %q,
      "server_port": %d,
      "version": "5"
    },
    {
      "type": "direct",
      "tag": "direct"
    }
  ],
  "route": {
    "auto_detect_interface": true,
    "final": "socks-out",
    "rules": [
      {
        "process_name": ["olcrtc.exe", "librertc-client.exe", "librertc-net-service.exe"],
        "outbound": "direct"
      },
      {
        "ip_cidr": ["127.0.0.0/8", "::1/128"],
        "outbound": "direct"
      },
      {
        "action": "sniff"
      },
      {
        "protocol": "dns",
        "action": "hijack-dns"
      },
      {
        "ip_is_private": true,
        "outbound": "direct"
      },
      {
        "ip_version": 6,
        "action": "reject"
      },
      {
        "network": "udp",
        "action": "reject"
      }
    ]
  }
}`, dnsServer, request.SocksHost, request.SocksPort)
	options, err := singjson.UnmarshalExtendedContext[option.Options](singBoxContext(context.Background()), []byte(raw))
	if err != nil {
		return option.Options{}, fmt.Errorf("decode sing-box config: %w", err)
	}
	return options, nil
}

func buildProxyOptions(request StartRequest) (option.Options, error) {
	if request.SocksHost == "" {
		request.SocksHost = "127.0.0.1"
	}
	if request.ListenHost == "" {
		request.ListenHost = "127.0.0.1"
	}
	if request.SocksPort <= 0 || request.SocksPort > 65535 {
		return option.Options{}, fmt.Errorf("invalid SOCKS port: %d", request.SocksPort)
	}
	if request.ListenPort <= 0 || request.ListenPort > 65535 {
		return option.Options{}, fmt.Errorf("invalid proxy listen port: %d", request.ListenPort)
	}
	raw := fmt.Sprintf(`{
  "log": {
    "level": "warn",
    "timestamp": true
  },
  "inbounds": [
    {
      "type": "mixed",
      "tag": "mixed-in",
      "listen": %q,
      "listen_port": %d
    }
  ],
  "outbounds": [
    {
      "type": "socks",
      "tag": "socks-out",
      "server": %q,
      "server_port": %d,
      "version": "5"
    },
    {
      "type": "direct",
      "tag": "direct"
    }
  ],
  "route": {
    "final": "socks-out",
    "rules": [
      {
        "ip_version": 6,
        "action": "reject"
      },
      {
        "network": "udp",
        "action": "reject"
      }
    ]
  }
}`, request.ListenHost, request.ListenPort, request.SocksHost, request.SocksPort)
	options, err := singjson.UnmarshalExtendedContext[option.Options](singBoxContext(context.Background()), []byte(raw))
	if err != nil {
		return option.Options{}, fmt.Errorf("decode sing-box proxy config: %w", err)
	}
	return options, nil
}

func singBoxContext(ctx context.Context) context.Context {
	inboundRegistry := inbound.NewRegistry()
	mixed.RegisterInbound(inboundRegistry)
	tun.RegisterInbound(inboundRegistry)

	outboundRegistry := outbound.NewRegistry()
	direct.RegisterOutbound(outboundRegistry)
	socks.RegisterOutbound(outboundRegistry)

	dnsRegistry := dns.NewTransportRegistry()
	local.RegisterTransport(dnsRegistry)
	transport.RegisterHTTPS(dnsRegistry)

	return box.Context(
		ctx,
		inboundRegistry,
		outboundRegistry,
		endpoint.NewRegistry(),
		dnsRegistry,
		boxservice.NewRegistry(),
	)
}

func dnsServerHost(value string) string {
	if value == "" {
		return "1.1.1.1"
	}
	for i, ch := range value {
		if ch == ':' {
			return value[:i]
		}
	}
	return value
}

func run(ctx context.Context) error {
	engine := &singBoxEngine{status: EngineStatus{Message: "ready"}}
	server := &http.Server{Addr: listenAddr, Handler: routes(engine)}

	go func() {
		<-ctx.Done()
		shutdownCtx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		_ = server.Shutdown(shutdownCtx)
	}()

	log.Printf("LibreRTC net service listening on http://%s", listenAddr)
	err := server.ListenAndServe()
	if errors.Is(err, http.ErrServerClosed) {
		return nil
	}
	return err
}

func routes(engine Engine) http.Handler {
	mux := http.NewServeMux()
	mux.HandleFunc("/health", func(writer http.ResponseWriter, request *http.Request) {
		writeJSON(writer, map[string]string{"status": "ok"})
	})
	mux.HandleFunc("/status", func(writer http.ResponseWriter, request *http.Request) {
		writeJSON(writer, engine.Status())
	})
	mux.HandleFunc("/start", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodPost {
			http.Error(writer, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		var payload StartRequest
		if err := stdjson.NewDecoder(request.Body).Decode(&payload); err != nil {
			http.Error(writer, err.Error(), http.StatusBadRequest)
			return
		}
		if err := engine.Start(request.Context(), payload); err != nil {
			http.Error(writer, err.Error(), http.StatusBadGateway)
			return
		}
		writeJSON(writer, engine.Status())
	})
	mux.HandleFunc("/stop", func(writer http.ResponseWriter, request *http.Request) {
		if request.Method != http.MethodPost {
			http.Error(writer, "method not allowed", http.StatusMethodNotAllowed)
			return
		}
		if err := engine.Stop(request.Context()); err != nil {
			http.Error(writer, err.Error(), http.StatusBadGateway)
			return
		}
		writeJSON(writer, engine.Status())
	})
	return mux
}

func writeJSON(writer http.ResponseWriter, value any) {
	writer.Header().Set("Content-Type", "application/json")
	_ = stdjson.NewEncoder(writer).Encode(value)
}

func installService() error {
	exe, err := os.Executable()
	if err != nil {
		return err
	}
	manager, err := mgr.Connect()
	if err != nil {
		return err
	}
	defer manager.Disconnect()

	service, err := manager.OpenService(serviceName)
	if err == nil {
		defer service.Close()
		return startInstalledService(service)
	}

	service, err = manager.CreateService(serviceName, filepath.Clean(exe), mgr.Config{
		DisplayName: displayName,
		StartType:   mgr.StartAutomatic,
	}, "")
	if err != nil {
		return err
	}
	defer service.Close()
	return startInstalledService(service)
}

func startInstalledService(service *mgr.Service) error {
	status, err := service.Query()
	if err == nil && status.State == svc.Running {
		return nil
	}
	err = service.Start()
	if err != nil && !errors.Is(err, windows.ERROR_SERVICE_ALREADY_RUNNING) {
		return err
	}
	return nil
}

func uninstallService() error {
	manager, err := mgr.Connect()
	if err != nil {
		return err
	}
	defer manager.Disconnect()
	service, err := manager.OpenService(serviceName)
	if err != nil {
		return err
	}
	defer service.Close()
	return service.Delete()
}

func must(err error) {
	if err != nil {
		log.Fatal(err)
	}
}
