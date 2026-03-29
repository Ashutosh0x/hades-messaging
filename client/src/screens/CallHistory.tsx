import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { invoke } from '@tauri-apps/api/core';
import { listen, Event } from '@tauri-apps/api/event';
import './CallHistory.css';
import { useTranslation } from 'react-i18next';
import { ArrowLeft, Phone, Video, Trash2, ICON_SIZE } from '../ui/icons';

type CallState = 'idle' | 'initiating' | 'calling' | 'incoming' | 'ended';

interface CallDetails {
  id: string;
  contact_id: string;
  contact_name: string;
  type: 'voice' | 'video';
  start_time: string;
  duration: number; // seconds
  status: 'initiated' | 'connected' | 'ended' | 'missed';
  direction: 'incoming' | 'outgoing';
  network: 'wifi' | 'p2p' | 'websocket';
}

interface CallEvent {
  id: string;
  status: 'initiated' | 'connected' | 'ended' | 'missed';
  duration: number; // For ended calls
}

function formatDuration(seconds: number) {
  const m = Math.floor(seconds / 60);
  const s = seconds % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

export default function CallHistory() {
  const navigate = useNavigate();
  const { t } = useTranslation();
  
  const [calls, setCalls] = useState<CallDetails[]>([]);
  const [status, setStatus] = useState<CallState>('idle');
  const [activeCallId, setActiveCallId] = useState<string | null>(null);
  const [screen, setScreen] = useState<'idle' | 'outgoing' | 'incoming' | 'call_ended'>('idle');

  useEffect(() => {
    if (screen === 'idle') {
      loadCallHistory();
    }
  }, [screen]);

  // Handle live status updates from Tauri back-end
  useEffect(() => {
    const setupListener = async () => {
      const unlisten = await listen<CallEvent>('call_status', (event: Event<CallEvent>) => {
        const { id, status: newStatus, duration } = event.payload;
        
        setCalls(prev => prev.map(c => 
          c.id === id ? { ...c, status: newStatus, duration: newStatus === 'ended' ? duration : 0 } : c
        ));
        
        if (newStatus === 'ended') {
          setScreen('call_ended');
          setTimeout(() => setScreen('idle'), 2000);
          setActiveCallId(null);
        } else if (newStatus === 'connected') {
          // Just update state if we are tracking this connection
        }
      });
      return unlisten;
    };
    
    let unlistenFn: any;
    setupListener().then(fn => unlistenFn = fn);
    return () => { if (unlistenFn) unlistenFn(); };
  }, []);

  const loadCallHistory = async () => {
    try {
      // Actually fetch from Rust
      const history = await invoke<CallDetails[]>('get_call_history');
      setCalls(history);
      setScreen('idle');
    } catch (e) {
      console.error("Failed to load call history", e);
    }
  };

  const clearHistory = async () => {
    // Left empty for backend implementation later
    setCalls([]);
  };

  const startCall = async (contactId: string, hasVideo: boolean) => {
    setScreen('outgoing');
    try {
      const callId = await invoke<string>('initiate_call', { contactId, hasVideo });
      setActiveCallId(callId);
    } catch (e) {
      console.error("Failed to start call:", e);
      setScreen('idle');
    }
  };

  const acceptCall = async (callId: string, conversationId: string) => {
    try {
      await invoke('accept_call', { callId, conversationId, iceCandidates: [] });
    } catch (e) {
      console.error("Accept call failed", e);
    }
  };

  const endCall = async (callId: string) => {
    try {
      await invoke('end_call', { callId, duration: 0 }); // Hardcoded duration placeholder
    } catch (e) {
      console.error("End call failed", e);
    }
  };

  return (
    <div className="call-history-screen calls-screen">
      <div className="call-history-header">
        <button className="back-btn" onClick={() => navigate(-1)} aria-label={t('common.back')}>
          <ArrowLeft size={ICON_SIZE.md} color="var(--text-primary)" />
        </button>
        <h1 className="call-history-title">Calls</h1>
      </div>

      <div className="calls-stats">
        <div className="stat">
          <span className="stat-value">{calls.filter(c => c.status === 'missed').length}</span>
          <span className="stat-label">Missed</span>
        </div>
        <div className="stat">
          <span className="stat-value">
            {calls.filter(c => c.status === 'initiated').length}
          </span>
          <span className="stat-label">
            Active calls
          </span>
        </div>
      </div>
      
      {activeCallId && (
        <div className="active-call-banner">
          Connecting to peer (ID: {activeCallId.slice(0, 8)}...)
        </div>
      )}

      <div className="call-history-list call-list">
        {calls.length === 0 ? (
          <div className="call-empty-state">
            <Phone size={48} color="var(--text-muted)" />
            <h3>No calls yet</h3>
          </div>
        ) : (
          calls.map(call => (
            <div
              key={call.id}
              className={`call-row ${call.status === 'missed' ? 'missed' : ''}`}
            >
              <div className="call-icon">
                {call.type === 'video' ? <Video size={16}/> : <Phone size={16}/>}
              </div>
              <div className="call-info">
                <div className="call-main">
                  <span className="call-name">{call.contact_name || call.contact_id.substring(0, 8)}</span>
                  <span className={`call-direction ${call.direction}`}>
                    {call.direction === 'incoming' ? 'Incoming call' : 'Outgoing call'}
                  </span>
                </div>
                <div className="call-meta">
                  <span>{new Date(Number(call.start_time) * 1000).toLocaleString([], {hour: 'numeric', minute: '2-digit'})}</span>
                  {call.duration > 0 && (
                    <span className="call-duration"> · {formatDuration(call.duration)}</span>
                  )}
                  <span className="call-network"> · {call.network}</span>
                </div>
              </div>
              <div className="call-status-pill">
                {call.status === 'initiated' && <span className="pill initiating">Initiating...</span>}
                {call.status === 'connected' && <span className="pill connected">Live</span>}
                {call.status === 'ended' && <span className="pill ended">Ended</span>}
              </div>
            </div>
          ))
        )}
      </div>

       {calls.length > 0 && (
        <div className="call-history-footer">
          <button className="clear-history-btn" onClick={clearHistory}>
            <Trash2 size={ICON_SIZE.sm} />
            {t('calls.clearHistory')}
          </button>
        </div>
      )}
    </div>
  );
}
