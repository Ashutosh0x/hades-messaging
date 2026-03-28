import { Routes, Route } from 'react-router-dom'
import ChatList from './screens/ChatList'
import Conversation from './screens/Conversation'
import ToastContainer from './components/ToastContainer'
import Onboarding from './screens/Onboarding'
import Settings from './screens/Settings'
import ProfileSettings from './screens/ProfileSettings'
import SecurityDetails from './screens/SecurityDetails'
import IncomingCall from './screens/IncomingCall'
import OutgoingCall from './screens/OutgoingCall'
import VoiceCall from './screens/VoiceCall'
import VideoCall from './screens/VideoCall'
import CallHistory from './screens/CallHistory'
import Contacts from './screens/Contacts'
import AddContact from './screens/AddContact'
import RecoveryPhrase from './screens/RecoveryPhrase'
import Wallet from './screens/Wallet'
import WalletSend from './screens/WalletSend'
import WalletReceive from './screens/WalletReceive'
import WalletHistory from './screens/WalletHistory'

export default function App() {
  return (
    <>
      <Routes>
        <Route path="/" element={<ChatList />} />
        <Route path="/conversation/:conversationId" element={<Conversation />} />
        <Route path="/onboarding" element={<Onboarding />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/profile" element={<ProfileSettings />} />
        <Route path="/security/:contactId" element={<SecurityDetails />} />
        <Route path="/incoming-call/:contactId" element={<IncomingCall />} />
        <Route path="/outgoing-call/:contactId" element={<OutgoingCall />} />
        <Route path="/voice-call/:contactId" element={<VoiceCall />} />
        <Route path="/video-call/:contactId" element={<VideoCall />} />
        <Route path="/call-history" element={<CallHistory />} />
        <Route path="/contacts" element={<Contacts />} />
        <Route path="/add-contact" element={<AddContact />} />
        <Route path="/recovery-phrase" element={<RecoveryPhrase />} />
        <Route path="/wallet" element={<Wallet />} />
        <Route path="/wallet/send" element={<WalletSend />} />
        <Route path="/wallet/receive" element={<WalletReceive />} />
        <Route path="/wallet/history" element={<WalletHistory />} />
      </Routes>
      <ToastContainer />
    </>
  )
}
