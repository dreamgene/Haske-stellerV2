import { AnimatePresence, motion } from "framer-motion"
import Header from "../components/Header"
import QRBlock from "../components/QRBlock"
import StatusIndicator from "../components/StatusIndicator"
import AccessPass from "../components/AccessPass"
import Countdown from "../components/Countdown"
import TrustBadge from "../components/TrustBadge"
import { useSessionRecovery } from "../hooks/useSessionRecovery"
import { usePaymentStore } from "../store/usePaymentStore"

export default function Checkout() {
  const { restartSession } = useSessionRecovery()
  const { status, paymentData, requestExpiresAt, error } = usePaymentStore()

  const walletHref = paymentData?.qr_payload
  const showRetry = status === "EXPIRED" || status === "ERROR"

  return (
    <div className="min-h-screen scroll-smooth bg-[#070B11]">
      <div className="mx-auto flex min-h-screen w-full max-w-md flex-col overflow-hidden bg-[radial-gradient(circle_at_top,rgba(13,200,255,0.14),transparent_30%),linear-gradient(180deg,#121821_0%,#11161F_55%,#0E141C_100%)] pb-28 shadow-[0_32px_120px_rgba(0,0,0,0.5)] sm:my-4 sm:min-h-[calc(100vh-2rem)] sm:rounded-[32px] sm:border sm:border-white/10">
        <div className="pointer-events-none absolute inset-x-0 top-0 h-40 bg-[radial-gradient(circle_at_top,rgba(255,255,255,0.08),transparent_60%)]" />

        <div className="relative flex-1 px-5 pb-6 pt-6 sm:px-6 sm:pt-6">
          <Header />

          <div className="mt-7">
            <AnimatePresence mode="wait" initial={false}>
              {status === "CONFIRMED" ? (
                <motion.div
                  key="access-pass"
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -8 }}
                  transition={{ duration: 0.26, delay: 0.1, ease: [0.22, 1, 0.36, 1] }}
                >
                  <AccessPass />
                </motion.div>
              ) : (
                <motion.div
                  key="checkout-flow"
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  exit={{ opacity: 0, y: -8 }}
                  transition={{ duration: 0.2, ease: "easeOut" }}
                  className="space-y-5"
                >
                  <div className="space-y-4">
                    <p className="text-[11px] font-bold uppercase tracking-[0.26em] text-[#6E86AA]">
                      Checkout
                    </p>
                    <div className="space-y-3">
                      <h2 className="max-w-[12ch] text-[2.6rem] font-black leading-[0.95] tracking-[-0.05em] text-white sm:max-w-none sm:text-[3.2rem]">
                        HASKE Demo Event
                      </h2>
                      <div className="rounded-[28px] border border-white/10 bg-[linear-gradient(180deg,rgba(255,255,255,0.05),rgba(255,255,255,0.02))] px-4 py-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]">
                        <div className="flex items-end justify-between gap-3">
                          <div>
                            <div className="text-[11px] font-bold uppercase tracking-[0.22em] text-slate-500">
                              Ticket price
                            </div>
                            <div className="mt-2 text-5xl font-black tracking-[-0.06em] text-white">
                              ₦10,000
                            </div>
                          </div>
                          <div className="rounded-full border border-white/10 bg-black/10 px-3 py-1.5 text-sm font-semibold text-slate-300">
                            or 10 XLM
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>

                  <QRBlock />

                  <StatusIndicator status={status} />

                  {error && status !== "ERROR" && (
                    <div className="rounded-[22px] border border-white/10 bg-white/[0.03] px-4 py-3 text-sm leading-6 text-slate-300">
                      {error}
                    </div>
                  )}

                  <Countdown expiresAt={requestExpiresAt} prefix="Session expires in" />

                  <TrustBadge className="justify-start pt-1" />
                </motion.div>
              )}
            </AnimatePresence>
          </div>
        </div>

        {status !== "CONFIRMED" && (
          <div className="sticky bottom-0 left-0 right-0 border-t border-white/10 bg-[linear-gradient(180deg,rgba(17,22,31,0.84),rgba(17,22,31,0.97))] px-5 pb-[calc(env(safe-area-inset-bottom)+1rem)] pt-3 backdrop-blur-xl sm:px-6">
            <div className="space-y-3">
              <button
                type="button"
                onClick={() => {
                  if (walletHref) {
                    window.location.href = walletHref
                  }
                }}
                disabled={!walletHref}
                className="flex min-h-14 w-full items-center justify-center rounded-[22px] bg-[linear-gradient(180deg,#45E1FF,#00BDEB)] px-5 py-4 text-base font-extrabold uppercase tracking-[0.1em] text-[#071018] shadow-[0_16px_40px_rgba(0,189,235,0.28)] transition hover:brightness-110 disabled:cursor-not-allowed disabled:opacity-40"
              >
                Open in Wallet
              </button>

              {showRetry && (
                <button
                  type="button"
                  onClick={restartSession}
                  className="flex min-h-14 w-full items-center justify-center rounded-[22px] border border-white/10 bg-white/[0.04] px-5 py-4 text-base font-bold uppercase tracking-[0.08em] text-white transition hover:bg-white/[0.08]"
                >
                  {status === "EXPIRED" ? "Generate new pass" : "Retry"}
                </button>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
