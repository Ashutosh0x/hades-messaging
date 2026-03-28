# Hades Relay — NixOS Hardened Configuration
#
# Deploy with:
#   nixos-rebuild switch --flake .#hades-relay
#
# Hardened relay server with full disk encryption, AppArmor,
# Tor hidden service, Caddy reverse proxy, and zero persistent user data.

{ config, pkgs, ... }:

{
  imports = [
    ./hardware-configuration.nix
    <nixpkgs/nixos/modules/profiles/hardened.nix>
  ];

  # ── System ──────────────────────────────────────────────────────
  system.stateVersion = "24.11";

  boot = {
    loader.systemd-boot.enable = true;
    loader.efi.canTouchEfiVariables = true;

    # Full disk encryption (LUKS)
    initrd.luks.devices.root = {
      device = "/dev/nvme0n1p2";
      preLVM = true;
      allowDiscards = true;
    };

    kernelParams = [
      "slab_nomerge"
      "init_on_alloc=1"
      "init_on_free=1"
      "page_alloc.shuffle=1"
      "lockdown=confidentiality"
    ];
  };

  # ── Packages (minimal surface) ─────────────────────────────────
  environment.systemPackages = with pkgs; [
    hades-relay
    tor
    coturn
    htop
    tmux
  ];

  # ── Networking ─────────────────────────────────────────────────
  networking = {
    hostName = "hades-relay";

    firewall = {
      enable = true;
      allowedTCPPorts = [ 443 22 ];
      allowedUDPPorts = [ ];
    };

    nameservers = [ "127.0.0.1" ];
  };

  # ── Security ───────────────────────────────────────────────────
  security = {
    lockKernelModules = true;
    protectKernelImage = true;
    allowSimultaneousMultithreading = false;
    forcePageTableIsolation = true;

    apparmor.enable = true;

    pam.loginLimits = [
      { domain = "*"; type = "hard"; item = "core"; value = "0"; }
    ];
  };

  # ── Hades Relay Service ────────────────────────────────────────
  systemd.services.hades-relay = {
    description = "Hades Messaging Relay";
    after = [ "network.target" "tor.service" ];
    wantedBy = [ "multi-user.target" ];

    serviceConfig = {
      Type = "simple";
      ExecStart = "/opt/hades/bin/hades-relay";
      User = "hades";
      Group = "hades";
      Restart = "always";
      RestartSec = 5;

      # Systemd hardening
      DynamicUser = true;
      ProtectSystem = "strict";
      ProtectHome = true;
      PrivateTmp = true;
      NoNewPrivileges = true;
      PrivateDevices = true;
      ProtectKernelTunables = true;
      ProtectKernelModules = true;
      ProtectControlGroups = true;
      MemoryDenyWriteExecute = true;
      RestrictNamespaces = true;
      RestrictRealtime = true;
      LockPersonality = true;
      SystemCallFilter = [ "@system-service" ];
      CapabilityBoundingSet = "";

      # Resource limits
      LimitNOFILE = 65536;
      MemoryMax = "512M";
    };

    environment = {
      RUST_LOG = "info";
      RELAY_BIND = "127.0.0.1";
      RELAY_PORT = "8443";
    };
  };

  users.users.hades = {
    isSystemUser = true;
    group = "hades";
    home = "/var/lib/hades";
    createHome = true;
  };
  users.groups.hades = {};

  # ── Caddy reverse proxy with automatic HTTPS ──────────────────
  services.caddy = {
    enable = true;
    virtualHosts."relay.hades.im" = {
      extraConfig = ''
        reverse_proxy localhost:8443

        header {
          Strict-Transport-Security "max-age=31536000; includeSubDomains; preload"
          X-Content-Type-Options nosniff
          X-Frame-Options DENY
          Content-Security-Policy "default-src 'none'"
          -Server
        }
      '';
    };
  };

  # ── Tor Configuration ─────────────────────────────────────────
  services.tor = {
    enable = true;
    client.enable = false;

    relay = {
      enable = true;
      role = "bridge";
      port = 9001;
      nickname = "HadesRelay";
      contactInfo = "admin@hades.im";
      accountingMax = "10 TB";
      accountingStart = "month 1 00:00";
    };

    settings = {
      HiddenServiceDir = "/var/lib/tor/hades-service";
      HiddenServicePort = "443 127.0.0.1:8443";
      HiddenServiceVersion = 3;

      HiddenServiceEnableIntroDoSDefense = true;
      HiddenServicePoWDefensesEnabled = true;
      HiddenServicePoWQueueRate = 250;
      HiddenServicePoWQueueBurst = 2500;
      HiddenServiceNumIntroductionPoints = 10;
    };
  };

  # ── TURN Server (E2EE Voice/Video) ─────────────────────────────
  services.coturn = {
    enable = true;
    listening-port = 3478;
    tls-listening-port = 5349;
    use-auth-secret = true;
    static-auth-secret-file = "/var/lib/hades/coturn-secret";
    realm = "turn.hades.onion";
    no-cli = true;
    no-tcp-relay = true;
    denied-peer-ip = [ "0.0.0.0-0.255.255.255" "10.0.0.0-10.255.255.255" ];
  };

  # ── SSH hardening ──────────────────────────────────────────────
  services.openssh = {
    enable = true;
    settings = {
      PasswordAuthentication = false;
      PermitRootLogin = "no";
      KbdInteractiveAuthentication = false;
    };
  };

  # ── Fail2ban ───────────────────────────────────────────────────
  services.fail2ban.enable = true;

  # ── Monitoring ─────────────────────────────────────────────────
  services.prometheus.exporters.node = {
    enable = true;
    enabledCollectors = [ "systemd" "processes" ];
  };

  # ── Auto-updates ───────────────────────────────────────────────
  system.autoUpgrade = {
    enable = true;
    allowReboot = false;
  };

  # ── Logging (minimal) ─────────────────────────────────────────
  services.journald.extraConfig = ''
    SystemMaxUse=100M
    MaxRetentionSec=7d
  '';
}
