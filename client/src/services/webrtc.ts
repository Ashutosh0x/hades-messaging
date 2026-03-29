import { EventEmitter } from 'events';

export interface CallConfig {
  iceServers: RTCIceServer[];
  stunServers: RTCIceServer[];
}

export interface CallMetadata {
  callId: string;
  contactId: string;
  contactName: string;
  type: 'voice' | 'video';
  direction: 'incoming' | 'outgoing';
  startedAt: number;
}

export interface CallStats {
  packetLoss: number;
  jitter: number;
  rtt: number;
  bitrate: number;
  resolution: string;
  fps: number;
}

export class WebRTCCall extends EventEmitter {
  private peerConnection: RTCPeerConnection | null = null;
  private localStream: MediaStream | null = null;
  private remoteStream: MediaStream | null = null;
  private dataChannel: RTCDataChannel | null = null;
  private callMetadata: CallMetadata | null = null;
  private statsInterval: number | null = null;
  private config: CallConfig;

  constructor(config: CallConfig) {
    super();
    this.config = config;
  }

  async startCall(
    contactId: string,
    contactName: string,
    type: 'voice' | 'video'
  ): Promise<CallMetadata> {
    const callId = crypto.randomUUID();

    this.callMetadata = {
      callId,
      contactId,
      contactName,
      type,
      direction: 'outgoing',
      startedAt: Date.now(),
    };

    // Create peer connection
    this.peerConnection = new RTCPeerConnection({
      iceServers: [...this.config.stunServers, ...this.config.iceServers],
    });

    // Set up event handlers
    this.setupPeerConnectionHandlers();

    // Get local media stream
    await this.getLocalMedia(type);

    // Add local tracks to peer connection
    this.localStream?.getTracks().forEach((track) => {
      this.peerConnection?.addTrack(track, this.localStream!);
    });

    // Create data channel for signaling metadata
    this.dataChannel = this.peerConnection.createDataChannel('call-signaling');
    this.setupDataChannel();

    // Create offer
    const offer = await this.peerConnection.createOffer();
    await this.peerConnection.setLocalDescription(offer);

    // Emit offer for signaling (will be sent via WebSocket to relay)
    this.emit('signaling-offer', {
      callId,
      contactId,
      sdp: offer.sdp,
      type: this.callMetadata.type,
    });

    return this.callMetadata;
  }

  async receiveCall(
    callId: string,
    contactId: string,
    contactName: string,
    type: 'voice' | 'video',
    sdp: string
  ): Promise<void> {
    this.callMetadata = {
      callId,
      contactId,
      contactName,
      type,
      direction: 'incoming',
      startedAt: Date.now(),
    };

    // Create peer connection
    this.peerConnection = new RTCPeerConnection({
      iceServers: [...this.config.stunServers, ...this.config.iceServers],
    });

    this.setupPeerConnectionHandlers();

    // Set remote description (offer)
    await this.peerConnection.setRemoteDescription(
      new RTCSessionDescription({ type: 'offer', sdp })
    );

    // Get local media
    await this.getLocalMedia(type);

    // Add local tracks
    this.localStream?.getTracks().forEach((track) => {
      this.peerConnection?.addTrack(track, this.localStream!);
    });

    // Create answer
    const answer = await this.peerConnection.createAnswer();
    await this.peerConnection.setLocalDescription(answer);

    // Emit answer for signaling
    this.emit('signaling-answer', {
      callId,
      contactId,
      sdp: answer.sdp,
    });
  }

  async handleAnswer(sdp: string): Promise<void> {
    if (!this.peerConnection) return;

    await this.peerConnection.setRemoteDescription(
      new RTCSessionDescription({ type: 'answer', sdp })
    );
  }

  async handleIceCandidate(candidate: RTCIceCandidateInit): Promise<void> {
    if (!this.peerConnection) return;

    try {
      await this.peerConnection.addIceCandidate(candidate);
    } catch (err) {
      console.error('Failed to add ICE candidate:', err);
    }
  }

  async handleSignalingMessage(
    message: { type: string; sdp?: string; candidate?: RTCIceCandidateInit }
  ): Promise<void> {
    if (!this.peerConnection) return;

    if (message.type === 'answer' && message.sdp) {
      await this.handleAnswer(message.sdp);
    } else if (message.type === 'candidate' && message.candidate) {
      await this.handleIceCandidate(message.candidate);
    }
  }

  muteAudio(muted: boolean): void {
    this.localStream?.getAudioTracks().forEach((track) => {
      track.enabled = !muted;
    });
    this.emit('audio-muted', muted);
  }

  muteVideo(muted: boolean): void {
    this.localStream?.getVideoTracks().forEach((track) => {
      track.enabled = !muted;
    });
    this.emit('video-muted', muted);
  }

  switchCamera(): void {
    // Implementation for mobile camera switching
    this.emit('camera-switched');
  }

  async endCall(): Promise<void> {
    // Stop local stream
    this.localStream?.getTracks().forEach((track) => track.stop());
    this.localStream = null;

    // Close peer connection
    this.peerConnection?.close();
    this.peerConnection = null;

    // Close data channel
    this.dataChannel?.close();
    this.dataChannel = null;

    // Stop stats collection
    if (this.statsInterval) {
      window.clearInterval(this.statsInterval);
      this.statsInterval = null;
    }

    this.emit('call-ended', this.callMetadata);
    this.callMetadata = null;
  }

  getRemoteStream(): MediaStream | null {
    return this.remoteStream;
  }

  getCallMetadata(): CallMetadata | null {
    return this.callMetadata;
  }

  getDuration(): number {
    if (!this.callMetadata) return 0;
    return Math.floor((Date.now() - this.callMetadata.startedAt) / 1000);
  }

  private async getLocalMedia(type: 'voice' | 'video'): Promise<void> {
    const constraints: MediaStreamConstraints = {
      audio: {
        echoCancellation: true,
        noiseSuppression: true,
        autoGainControl: true,
      },
      video: type === 'video' ? {
        width: { ideal: 1280 },
        height: { ideal: 720 },
        facingMode: 'user',
      } : false,
    };

    try {
      this.localStream = await navigator.mediaDevices.getUserMedia(constraints);
      this.emit('local-stream-ready', this.localStream);
    } catch (err) {
      this.emit('media-error', err);
      throw err;
    }
  }

  private setupPeerConnectionHandlers(): void {
    if (!this.peerConnection) return;

    this.peerConnection.ontrack = (event) => {
      if (!this.remoteStream) {
        this.remoteStream = new MediaStream();
      }
      event.streams[0]?.getTracks().forEach((track) => {
        this.remoteStream?.addTrack(track);
      });
      this.emit('remote-stream-ready', this.remoteStream);
    };

    this.peerConnection.onicecandidate = (event) => {
      if (event.candidate) {
        this.emit('signaling-candidate', {
          callId: this.callMetadata?.callId,
          candidate: event.candidate,
        });
      }
    };

    this.peerConnection.oniceconnectionstatechange = () => {
      this.emit('ice-state-change', this.peerConnection?.iceConnectionState);
    };

    this.peerConnection.onconnectionstatechange = () => {
      this.emit('connection-state-change', this.peerConnection?.connectionState);

      if (this.peerConnection?.connectionState === 'connected') {
        this.startStatsCollection();
      }
    };
  }

  private setupDataChannel(): void {
    if (!this.dataChannel) return;

    this.dataChannel.onopen = () => {
      this.emit('data-channel-open');
    };

    this.dataChannel.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.emit('data-channel-message', data);
      } catch (err) {
        console.error('Failed to parse data channel message:', err);
      }
    };

    this.dataChannel.onclose = () => {
      this.emit('data-channel-closed');
    };
  }

  private startStatsCollection(): void {
    if (!this.peerConnection) return;

    this.statsInterval = window.setInterval(async () => {
      try {
        const stats = await this.peerConnection!.getStats();
        const callStats = this.parseStats(stats);
        this.emit('call-stats', callStats);
      } catch (err) {
        console.error('Failed to collect call stats:', err);
      }
    }, 3000);
  }

  private parseStats(stats: RTCStatsReport): CallStats {
    let packetLoss = 0;
    let jitter = 0;
    let rtt = 0;
    let bitrate = 0;
    let resolution = '0x0';
    let fps = 0;

    stats.forEach((report) => {
      if (report.type === 'inbound-rtp') {
        packetLoss = report.packetsLost || 0;
        jitter = report.jitter || 0;
        bitrate = report.bytesReceived || 0;
      } else if (report.type === 'remote-inbound-rtp') {
        rtt = report.roundTripTime ? report.roundTripTime * 1000 : 0;
      } else if (report.type === 'track' && report.kind === 'video') {
        resolution = `${report.frameWidth || 0}x${report.frameHeight || 0}`;
        fps = report.framesPerSecond || 0;
      }
    });

    return { packetLoss, jitter, rtt, bitrate, resolution, fps };
  }
}

// Default configuration
export const DEFAULT_CALL_CONFIG: CallConfig = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
  ],
  stunServers: [
    { urls: 'stun:stun.l.google.com:19302' },
  ],
};
