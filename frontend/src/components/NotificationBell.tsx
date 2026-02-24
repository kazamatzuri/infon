import { useEffect, useState, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { api } from '../api/client';
import type { Notification } from '../api/client';

function timeAgo(dateStr: string): string {
  const now = Date.now();
  const utcStr = dateStr.endsWith('Z') || dateStr.includes('+') ? dateStr : dateStr + 'Z';
  const then = new Date(utcStr).getTime();
  const diffSec = Math.floor((now - then) / 1000);
  if (diffSec < 60) return 'just now';
  if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
  if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
  return `${Math.floor(diffSec / 86400)}d ago`;
}

export function NotificationBell() {
  const navigate = useNavigate();
  const [notifications, setNotifications] = useState<Notification[]>([]);
  const [unreadCount, setUnreadCount] = useState(0);
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let cancelled = false;
    const load = async () => {
      if (cancelled) return;
      try {
        const result = await api.listNotifications();
        if (!cancelled) {
          setNotifications(result.notifications);
          setUnreadCount(result.unread_count);
        }
      } catch {
        // Silently fail - notifications are non-critical
      }
    };
    load();
    const interval = setInterval(load, 30000);
    return () => { cancelled = true; clearInterval(interval); };
  }, []);

  // Close dropdown when clicking outside
  useEffect(() => {
    function handleClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, []);

  const handleNotificationClick = async (notification: Notification) => {
    if (!notification.read) {
      await api.markNotificationRead(notification.id);
      setUnreadCount(prev => Math.max(0, prev - 1));
      setNotifications(prev =>
        prev.map(n => n.id === notification.id ? { ...n, read: true } : n)
      );
    }

    // Navigate based on notification data
    if (notification.data) {
      try {
        const data = JSON.parse(notification.data);
        if (data.match_id) {
          navigate(`/matches/${data.match_id}`);
          setOpen(false);
        }
      } catch {
        // Ignore parse errors
      }
    }
  };

  return (
    <div ref={ref} style={{ position: 'relative' }}>
      <button
        onClick={() => setOpen(!open)}
        style={{
          background: 'transparent',
          border: 'none',
          cursor: 'pointer',
          padding: '4px',
          position: 'relative',
          display: 'flex',
          alignItems: 'center',
        }}
        title="Notifications"
      >
        {/* Bell SVG icon */}
        <svg
          width="20"
          height="20"
          viewBox="0 0 24 24"
          fill="none"
          stroke="#888"
          strokeWidth="2"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9" />
          <path d="M13.73 21a2 2 0 0 1-3.46 0" />
        </svg>

        {/* Unread count badge */}
        {unreadCount > 0 && (
          <span style={{
            position: 'absolute',
            top: '-2px',
            right: '-4px',
            background: '#e94560',
            color: '#fff',
            borderRadius: '10px',
            padding: '1px 5px',
            fontSize: '10px',
            fontWeight: 700,
            minWidth: '16px',
            textAlign: 'center',
            lineHeight: '14px',
          }}>
            {unreadCount > 99 ? '99+' : unreadCount}
          </span>
        )}
      </button>

      {/* Dropdown */}
      {open && (
        <div style={{
          position: 'absolute',
          top: '100%',
          right: 0,
          marginTop: '8px',
          width: '340px',
          maxHeight: '400px',
          overflowY: 'auto',
          background: '#1a1a2e',
          border: '1px solid #333',
          borderRadius: '8px',
          boxShadow: '0 4px 16px rgba(0,0,0,0.4)',
          zIndex: 1000,
        }}>
          <div style={{
            padding: '12px 16px',
            borderBottom: '1px solid #333',
            color: '#e0e0e0',
            fontWeight: 600,
            fontSize: '14px',
          }}>
            Notifications
          </div>

          {notifications.length === 0 ? (
            <div style={{
              padding: '24px 16px',
              textAlign: 'center',
              color: '#666',
              fontSize: '13px',
            }}>
              No notifications
            </div>
          ) : (
            notifications.map(n => (
              <div
                key={n.id}
                onClick={() => handleNotificationClick(n)}
                style={{
                  padding: '10px 16px',
                  borderBottom: '1px solid #16213e',
                  cursor: 'pointer',
                  background: n.read ? 'transparent' : '#16213e44',
                }}
                onMouseOver={e => (e.currentTarget.style.background = '#16213e')}
                onMouseOut={e => (e.currentTarget.style.background = n.read ? 'transparent' : '#16213e44')}
              >
                <div style={{
                  display: 'flex',
                  alignItems: 'flex-start',
                  gap: '8px',
                }}>
                  {!n.read && (
                    <div style={{
                      width: '6px',
                      height: '6px',
                      borderRadius: '50%',
                      background: '#16c79a',
                      marginTop: '6px',
                      flexShrink: 0,
                    }} />
                  )}
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div style={{
                      color: '#e0e0e0',
                      fontSize: '13px',
                      fontWeight: n.read ? 400 : 600,
                      marginBottom: '2px',
                    }}>
                      {n.title}
                    </div>
                    <div style={{
                      color: '#888',
                      fontSize: '12px',
                      marginBottom: '2px',
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}>
                      {n.message}
                    </div>
                    <div style={{ color: '#666', fontSize: '11px' }}>
                      {timeAgo(n.created_at)}
                    </div>
                  </div>
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}
