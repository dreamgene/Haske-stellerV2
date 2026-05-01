import { motion } from "framer-motion"
import QRCode from "react-qr-code"
import { usePaymentStore } from "../store/usePaymentStore"
import { QRBlockSkeleton } from "./LoadingSkeleton"

const qrBlockTransition = {
  duration: 0.22,
  ease: [0.22, 1, 0.36, 1],
}

export default function QRBlock() {
  const { paymentData, status, error } = usePaymentStore()

  if (!paymentData && status !== "ERROR") {
    return <QRBlockSkeleton />
  }

  if (!paymentData && status === "ERROR") {
    return (
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.18, ease: "easeOut" }}
        className="overflow-hidden rounded-[32px] border border-[#FF5A5A]/25 bg-[radial-gradient(circle_at_top,rgba(255,90,90,0.12),rgba(36,12,15,0.95)_60%)] p-5 shadow-[0_24px_80px_rgba(0,0,0,0.36)]"
      >
        <div className="flex min-h-[300px] flex-col items-center justify-center rounded-[28px] border border-[#FF5A5A]/20 bg-black/10 px-6 text-center">
          <div className="flex h-14 w-14 items-center justify-center rounded-full border border-[#FF5A5A]/25 bg-[#FF5A5A]/12 text-[#FF6F6F]">
            <svg
              viewBox="0 0 24 24"
              className="h-6 w-6"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.8"
              strokeLinecap="round"
              strokeLinejoin="round"
              aria-hidden="true"
            >
              <path d="M12 9v4" />
              <path d="m12 17 .01 0" />
              <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0Z" />
            </svg>
          </div>
          <p className="mt-5 text-lg font-bold tracking-tight text-white">
            Payment session unavailable
          </p>
          <p className="mt-2 max-w-[24ch] text-sm leading-6 text-slate-300">
            {error ?? "We could not generate a fresh payment QR right now."}
          </p>
          <p className="mt-4 text-xs font-semibold uppercase tracking-[0.16em] text-slate-500">
            Check backend connectivity and retry
          </p>
        </div>
      </motion.div>
    )
  }

  const isDetected = status === "DETECTED" || status === "CONFIRMED"

  return (
    <motion.div
      initial={false}
      animate={{
        opacity: isDetected ? 0.58 : 1,
        scale: isDetected ? 0.985 : 1,
        filter: isDetected ? "saturate(0.92)" : "saturate(1)",
      }}
      transition={{
        ...qrBlockTransition,
        duration: status === "DETECTED" ? 0.12 : qrBlockTransition.duration,
      }}
      className="overflow-hidden rounded-[32px] border border-white/10 bg-[radial-gradient(circle_at_top,rgba(255,255,255,0.08),rgba(255,255,255,0.02)_58%)] p-5 text-center shadow-[0_24px_80px_rgba(0,0,0,0.32)] sm:p-6"
    >
      <div className="mx-auto flex min-h-[224px] items-center justify-center rounded-[28px] bg-white p-4 shadow-[inset_0_1px_0_rgba(255,255,255,0.72)] sm:min-h-[240px]">
        {paymentData.qr_png ? (
          <img
            src={paymentData.qr_png}
            alt="Payment QR"
            className="mx-auto h-auto w-full max-w-[232px] object-contain sm:max-w-[240px]"
          />
        ) : (
          <div className="flex w-full justify-center">
            <QRCode value={paymentData.qr_payload} size={208} />
          </div>
        )}
      </div>

      <div className="mt-5 space-y-2 text-slate-100">
        <p className="text-base font-semibold tracking-tight text-white">
          Scan with any Lightning wallet
        </p>
        <p className="text-sm leading-6 text-slate-400">
          Fast settlement. Signed access issued after confirmation.
        </p>
      </div>
      <div className="mt-4 inline-flex items-center rounded-full border border-white/10 bg-white/[0.03] px-3 py-1.5 text-xs font-semibold uppercase tracking-[0.12em] text-slate-400">
        BOLT11 invoice
      </div>
    </motion.div>
  )
}
