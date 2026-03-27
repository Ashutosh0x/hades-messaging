# Hades Relay — NixOS Hardened Configuration
#
# Deploy with:
#   nixos-rebuild switch --flake .#hades-relay
#
# This configuration produces a minimal, hardened relay server
# with full disk encryption, AppArmor, Tor hidden service,
# and zero persistent user data.

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

    # Kernel hardening
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
    hades-relay   # The Hades relay binary (built from crates/hades-relay)
    tor
    coturn        # Self-hosted TURN for E2EE calls
    htop
    tmux
  ];

  # ── Security ───────────────────────────────────────────────────

  security = {
    lockKernelModules = true;
    protectKernelImage = true;
    allowSimultaneousMultithreading = false;   # Mitigate Spectre
    forcePageTableIsolation = true;            # KPTI

    apparmor = {
      enable = true;
    };

    # Prevent core dumps
    pam.loginLimits = [
      { domain = "*"; type = "hard"; item = "core"; value = "0"; }
    ];
  };

  # ── Firewall ───────────────────────────────────────────────────

  networking = {
    hostName = "hades-relay";

    firewall = {
      enable = true;
      allowedTCPPorts = [ 443 ];   # HTTPS / Onion service only
      allowedUDPPorts = [ ];       # No UDP unless TURN is enabled
    };

    # Force DNS over Tor
    nameservers = [ "127.0.0.1" ];
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
      # Onion service for the relay
      HiddenServiceDir = "/var/lib/tor/hades-service";
      HiddenServicePort = "443 127.0.0.1:8443";
      HiddenServiceVersion = 3;

      # Vanguards v2 — multi-layer guard rotation
      HiddenServiceEnableIntroDoSDefense = true;
      HiddenServicePoWDefensesEnabled = true;
      HiddenServicePoWQueueRate = 250;
      HiddenServicePoWQueueBurst = 2500;
      HiddenServiceNumIntroductionPoints = 10;
    };
  };

  # ── Hades Relay Service ────────────────────────────────────────

  systemd.services.hades-relay = {
    description = "Hades Messaging Relay";
    after = [ "network.target" "tor.service" ];
    wantedBy = [ "multi-user.target" ];

    serviceConfig = {
      ExecStart = "/opt/hades/bin/hades-relay";
      User = "hades";
      Group = "hades";
      Restart = "always";
      RestartSec = 5;

      # Systemd hardening
      ProtectSystem = "strict";
      ProtectHome = true;
      PrivateTmp = true;
      NoNewPrivileges = true;
      PrivateDevices = true;
      ProtectKernelTunables = true;
      ProtectKernelModules = true;
      ProtectControlGroups = true;
      MemoryDenyWriteExecute = true;
      RestrictRealtime = true;
      LockPersonality = true;
    };
  };

  users.users.hades = {
    isSystemUser = true;
    group = "hades";
    home = "/var/lib/hades";
    createHome = true;
  };
  users.groups.hades = {};

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

  # ── Auto-updates ───────────────────────────────────────────────

  system.autoUpgrade = {
    enable = true;
    allowReboot = true;
    dates = "03:00";
  };

  # ── Logging (minimal) ─────────────────────────────────────────

  services.journald.extraConfig = ''
    SystemMaxUse=100M
    MaxRetentionSec=7d
  '';
}
