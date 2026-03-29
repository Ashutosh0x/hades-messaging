import React, { useEffect, useRef, useState } from 'react';
import { motion } from 'framer-motion';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { WebRTCCall, CallMetadata, CallStats, DEFAULT_CALL_CONFIG } from '../services/webrtc';

export const CallScreen: React.FC = () => {
  const navigate = useNavigate();
  const [params] = useSearchParams();

  const callRef = useRef<WebRTCCall | null>(null);
  const localVideoRef = useRef<HTMLVideoElement>(null);
  const remoteVideoRef = useRef<HTMLVideoElement>(null);

  const [callMetadata, setCallMetadata] = useState<CallMetadata | null>(null);
  const [isMuted, setIsMuted] = useState(false);
  const [isVideoOff, setIsVideoOff] = useState(false);
  const [isSpeakerOn, setIsSpeakerOn] = useState(false);
  const [callDuration, setCallDuration] = useState(0);
  const [callStats, setCallStats] = useState<CallStats | null>(null);
  const [callState, setCallState] = useState<'connecting' | 'connected' | 'ending'>('connecting');

  useEffect(() => {
    const call = new WebRTCCall(DEFAULT_CALL_CONFIG);
    callRef.current = call;

    // Set up event handlers
    call.on('local-stream-ready', (stream: MediaStream) => {
      if (localVideoRef.current) {
        localVideoRef.current.srcObject = stream;
      }
    });

    call.on('remote-stream-ready', (stream: MediaStream) => {
      if (remoteVideoRef.current) {
        remoteVideoRef.current.srcObject = stream;
      }
      setCallState('connected');
    });

    call.on('signaling-offer', async (data: any) => {
      // Send offer via Tauri backend
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('send_call_offer', {
          callId: data.callId,
          recipientId: data.contactId,
          callType: data.type,
          sdp: data.sdp,
        });
      } catch (err) {
        console.error('Failed to send offer:', err);
      }
    });

    call.on('signaling-answer', async (data: any) => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('send_call_answer', {
          callId: data.callId,
          callerId: data.contactId,
          sdp: data.sdp,
        });
      } catch (err) {
        console.error('Failed to send answer:', err);
      }
    });

    call.on('signaling-candidate', async (data: any) => {
      try {
        const { invoke } = await import('@tauri-apps/api/core');
        await invoke('send_call_ice_candidate', {
          callId: data.callId,
          recipientId: data.contactId,
          candidate: data.candidate,
        });
      } catch (err) {
        console.error('Failed to send ICE candidate:', err);
      }
    });

    call.on('call-stats', (stats: CallStats) => {
      setCallStats(stats);
    });

    call.on('call-ended', () => {
      setCallState('ending');
      setTimeout(() => navigate('/chats'), 1000);
    });

    // Start or receive call
    const contactId = params.get('contactId');
    const contactName = params.get('contactName');
    const type = (params.get('type') as 'voice' | 'video') || 'voice';
    const incoming = params.get('incoming') === 'true';
    const incomingSdp = params.get('sdp');

    if (contactId && contactName) {
      if (incoming && incomingSdp) {
        call.receiveCall(
          crypto.randomUUID(),
          contactId,
          contactName,
          type,
          incomingSdp
        ).catch(console.error);
      } else {
        call.startCall(contactId, contactName, type).then(setCallMetadata).catch(console.error);
      }
    }

    // Duration timer
    const timer = setInterval(() => {
      if (callState === 'connected') {
        setCallDuration((d) => d + 1);
      }
    }, 1000);

    return () => {
      clearInterval(timer);
      call.endCall().catch(console.error);
    };
  }, [navigate, params, callState]);

  const handleEndCall = () => {
    callRef.current?.endCall();
  };

  const handleToggleMute = () => {
    callRef.current?.muteAudio(!isMuted);
    setIsMuted(!isMuted);
  };

  const handleToggleVideo = () => {
    callRef.current?.muteVideo(!isVideoOff);
    setIsVideoOff(!isVideoOff);
  };

  const formatDuration = (secs: number) => {
    const mins = Math.floor(secs / 60);
    const s = secs % 60;
    return `${mins}:${s.toString().padStart(2, '0')}`;
  };

  return (
    <div className="call-screen">
      {/* Remote video (full screen) */}
      <video
        ref={remoteVideoRef}
        autoPlay
        playsInline
        className="remote-video"
      />

      {/* Local video (picture-in-picture) */}
      <video
        ref={localVideoRef}
        autoPlay
        playsInline
        muted
        className="local-video"
      />

      {/* Call info overlay */}
      <div className="call-overlay">
        <div className="call-info">
          <h2>{callMetadata?.contactName || 'Unknown'}</h2>
          <p className="call-type">{callMetadata?.type || 'Voice'} Call</p>
          <p className="call-duration">{formatDuration(callDuration)}</p>

          {callStats && (
            <div className="call-stats">
              <span>📶 {callStats.packetLoss < 5 ? 'Good' : 'Poor'}</span>
              <span>⏱️ {callStats.rtt}ms</span>
            </div>
          )}
        </div>

        {/* Call controls */}
        <div className="call-controls">
          <button
            className={`control-btn ${isMuted ? 'active' : ''}`}
            onClick={handleToggleMute}
          >
            {isMuted ? '🔇' : '🎤'}
          </button>

          {callMetadata?.type === 'video' && (
            <button
              className={`control-btn ${isVideoOff ? 'active' : ''}`}
              onClick={handleToggleVideo}
            >
              {isVideoOff ? '📷' : '📹'}
            </button>
          )}

          <button
            className={`control-btn ${isSpeakerOn ? 'active' : ''}`}
            onClick={() => setIsSpeakerOn(!isSpeakerOn)}
          >
            {isSpeakerOn ? '🔊' : '📱'}
          </button>

          <button className="control-btn end-call" onClick={handleEndCall}>
            📞
          </button>
        </div>
      </div>

      {/* Connecting state */}
      {callState === 'connecting' && (
        <motion.div
          className="connecting-overlay"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
        >
          <motion.div
            animate={{ scale: [1, 1.2, 1] }}
            transition={{ duration: 1.5, repeat: Infinity }}
            className="connecting-spinner"
          >
            ◈
          </motion.div>
          <p>Connecting...</p>
        </motion.div>
      )}
    </div>
  );
};
