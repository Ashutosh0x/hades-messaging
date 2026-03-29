import { create } from 'zustand';
import type {
  AccountInfo,
  BalanceResult,
  ChainId,
  GasEstimate,
  SendRequest,
  WalletTx,
} from '../types/wallet';

// Helper: try to invoke Tauri command, return fallback if not in Tauri context
async function tryInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T | null> {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    return await invoke<T>(cmd, args);
  } catch {
    return null;
  }
}

interface WalletState {
  // State
  initialized: boolean;
  accounts: AccountInfo[];
  balances: Record<string, BalanceResult>;
  transactions: WalletTx[];
  mnemonic: string | null;
  loading: boolean;
  sending: boolean;
  error: string | null;
  selectedChain: ChainId;
  totalUsdValue: number;

  // Actions
  initWallet: () => Promise<void>;
  importWallet: (mnemonic: string) => Promise<void>;
  fetchAllBalances: () => Promise<void>;
  fetchBalance: (chain: ChainId) => Promise<void>;
  sendCrypto: (req: SendRequest) => Promise<{ txHash: string; explorerUrl: string; messageId?: string }>;
  estimateGas: (chain: ChainId, to: string, amount: string) => Promise<GasEstimate>;
  fetchTransactions: (chain?: ChainId) => Promise<void>;
  getAddress: (chain: ChainId) => Promise<string>;
  exportMnemonic: () => Promise<string>;
  setSelectedChain: (chain: ChainId) => void;
  clearMnemonic: () => void;
}

export const useWalletStore = create<WalletState>((set, get) => ({
  initialized: false,
  accounts: [],
  balances: {},
  transactions: [],
  mnemonic: null,
  loading: false,
  sending: false,
  error: null,
  selectedChain: 'Ethereum',
  totalUsdValue: 0,

  initWallet: async () => {
    set({ loading: true, error: null });
    try {
      const result = await tryInvoke<{
        accounts: AccountInfo[];
        mnemonic: string | null;
      }>('wallet_init');

      if (result) {
        set({
          initialized: true,
          accounts: result.accounts,
          mnemonic: result.mnemonic,
          loading: false,
        });
        get().fetchAllBalances();
      } else {
        set({ loading: false });
      }
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  importWallet: async (mnemonic: string) => {
    set({ loading: true, error: null });
    try {
      const result = await tryInvoke<{
        accounts: AccountInfo[];
        mnemonic: string | null;
      }>('wallet_import', { mnemonic });

      if (result) {
        set({
          initialized: true,
          accounts: result.accounts,
          loading: false,
        });
        get().fetchAllBalances();
      } else {
        set({ loading: false });
      }
    } catch (err) {
      set({ error: String(err), loading: false });
    }
  },

  fetchAllBalances: async () => {
    try {
      const results = await tryInvoke<BalanceResult[]>('wallet_get_all_balances');
      if (!results) return;
      const balances: Record<string, BalanceResult> = {};
      let total = 0;

      for (const b of results) {
        balances[b.chain] = b;
        if (b.usdValue) total += b.usdValue;
      }

      set({ balances, totalUsdValue: total });
    } catch (err) {
      console.error('Failed to fetch balances:', err);
    }
  },

  fetchBalance: async (chain: ChainId) => {
    try {
      const result = await tryInvoke<BalanceResult>('wallet_get_balance', { chain });
      if (result) {
        set((state) => ({
          balances: { ...state.balances, [chain]: result },
        }));
      }
    } catch (err) {
      console.error(`Failed to fetch ${chain} balance:`, err);
    }
  },

  sendCrypto: async (req: SendRequest) => {
    set({ sending: true, error: null });
    try {
      const result = await tryInvoke<{
        txHash: string;
        explorerUrl: string;
        status: string;
        messageId: string | null;
      }>('wallet_send', { request: req });

      if (!result) throw new Error('Wallet not available in browser mode');

      get().fetchBalance(req.chain);
      get().fetchTransactions();

      set({ sending: false });

      return {
        txHash: result.txHash,
        explorerUrl: result.explorerUrl,
        messageId: result.messageId ?? undefined,
      };
    } catch (err) {
      set({ sending: false, error: String(err) });
      throw err;
    }
  },

  estimateGas: async (chain, to, amount) => {
    const result = await tryInvoke<GasEstimate>('wallet_estimate_gas', { chain, to, amount });
    if (!result) throw new Error('Wallet not available in browser mode');
    return result;
  },

  fetchTransactions: async (chain?: ChainId) => {
    try {
      const txs = await tryInvoke<WalletTx[]>('wallet_get_transactions', {
        chain: chain ?? null,
        limit: 100,
      });
      if (txs) set({ transactions: txs });
    } catch (err) {
      console.error('Failed to fetch transactions:', err);
    }
  },

  getAddress: async (chain: ChainId) => {
    const result = await tryInvoke<string>('wallet_get_address', { chain });
    return result ?? '';
  },

  exportMnemonic: async () => {
    const result = await tryInvoke<string>('wallet_export_mnemonic');
    return result ?? '';
  },

  setSelectedChain: (chain) => set({ selectedChain: chain }),
  clearMnemonic: () => set({ mnemonic: null }),
}));
