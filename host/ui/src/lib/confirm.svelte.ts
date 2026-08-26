import { getContext, setContext } from "svelte";

export interface ConfirmRequest {
  title: string;
  message: string;
  confirmLabel?: string;
  danger?: boolean;
}

interface PendingConfirm {
  req: ConfirmRequest;
  resolve: (v: boolean) => void;
}

export function createConfirmCtx() {
  const state = $state<{ pending: PendingConfirm | null }>({ pending: null });

  function requestConfirm(req: ConfirmRequest): Promise<boolean> {
    return new Promise<boolean>((resolve) => {
      state.pending = { req, resolve };
    });
  }

  function settle(v: boolean) {
    if (state.pending) {
      state.pending.resolve(v);
      state.pending = null;
    }
  }

  return { state, requestConfirm, settle };
}

export type ConfirmCtx = ReturnType<typeof createConfirmCtx>;

const CONFIRM_KEY = Symbol("confirm");

export function setConfirmCtx(ctx: ConfirmCtx) {
  setContext(CONFIRM_KEY, ctx);
}

export function getConfirmCtx(): ConfirmCtx {
  return getContext<ConfirmCtx>(CONFIRM_KEY);
}
