// Hades icon defaults — enforced project-wide.
import type { LucideProps } from 'lucide-react';

export const ICON_DEFAULTS: Partial<LucideProps> = {
  size: 20,
  strokeWidth: 1.75,
  color: 'currentColor',
  absoluteStrokeWidth: false,
} as const;

export const ICON_SIZE = {
  xs: 14,
  sm: 16,
  md: 20,
  lg: 24,
  xl: 32,
  xxl: 48,
} as const;

export const ICON_STROKE = {
  thin: 1.25,
  default: 1.75,
  bold: 2.25,
} as const;

// ── Navigation ──
export {
  MessageSquare, Settings, Search, Plus, ArrowLeft, X, MoreVertical,
  ChevronRight, ChevronLeft, ChevronDown, ChevronUp, ArrowRight, ArrowDown,
} from 'lucide-react';

// ── Security & Encryption ──
export {
  ShieldCheck, Shield, ShieldAlert, ShieldOff, ShieldX, ShieldPlus,
  Lock, LockKeyhole, LockOpen, Unlock, Key, KeyRound,
  Fingerprint, ScanFace, EyeOff, Eye,
  AlertTriangle, AlertOctagon, Ban,
} from 'lucide-react';

// ── Messaging ──
export {
  Send, SendHorizontal, Paperclip, Image, FileText, File, FileArchive,
  Mic, Camera, Smile, Reply, Forward, Copy, Trash2, Clipboard, Type,
  Clock, Timer, Check, CheckCheck, CircleAlert, RotateCcw, Pin, Bookmark,
} from 'lucide-react';

// ── Contacts & Identity ──
export {
  User, UserRound, UserPlus, UserMinus, UserCheck, UserX, Users,
  UserRoundSearch, QrCode, ScanLine, Hash, CircleUser, Contact,
} from 'lucide-react';

// ── Tor & Network ──
export {
  Globe, GlobeLock, Route, Waypoints, Activity,
  Wifi, WifiOff, Signal, SignalLow, SignalZero,
  RefreshCw, Zap, Server, Network, Radio, ArrowDownUp, CloudOff,
} from 'lucide-react';

// ── Settings ──
export {
  Bell, BellOff, BellRing, Monitor, Smartphone, Laptop, Tablet,
  Link, Unlink, ToggleLeft, ToggleRight, Sun, Moon,
  HelpCircle, Info, ExternalLink, Download, Upload, LogOut,
} from 'lucide-react';

// -- Emergency & Recovery --
export {
  Siren, Flame, Eraser, ScrollText, ClipboardCheck, CircleX, Delete,
} from 'lucide-react';

// ── Verification & Audit ──
export {
  Grid3x3, BadgeCheck, BadgeAlert, BadgeX, ClipboardList, FileSearch,
  Binary, Braces, ListChecks, CircleCheckBig, CircleDot, CircleDashed,
} from 'lucide-react';

// ── Onboarding ──
export {
  Loader, CheckCircle, Circle, Sparkles, DatabaseZap,
} from 'lucide-react';

// ── Calling ──
export {
  Phone, PhoneCall, PhoneIncoming, PhoneOutgoing, PhoneMissed, PhoneOff,
  Video, VideoOff, MicOff, Volume2, VolumeX,
  ScreenShare, ScreenShareOff, SwitchCamera, Ghost,
  Pause, Play, Grid2x2,
} from 'lucide-react';

export type { LucideProps, LucideIcon } from 'lucide-react';
