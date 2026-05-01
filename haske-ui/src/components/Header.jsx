export default function Header() {
  return (
    <div className="flex w-full items-start justify-between gap-4">
      <div className="space-y-1">
        <div className="text-[1.35rem] font-black tracking-[0.18em] text-white">
          HASKEpay <span className="text-[#F8C33C]">⚡</span>
        </div>
        <p className="text-xs font-medium tracking-[0.08em] text-slate-500">
          Offline-ready event access
        </p>
      </div>

      <span className="inline-flex min-h-10 items-center rounded-full border border-[#0CC8FF]/35 bg-[#0CC8FF]/10 px-4 py-2 text-[11px] font-bold uppercase tracking-[0.2em] text-[#48D9FF] shadow-[0_0_0_1px_rgba(12,200,255,0.08)]">
        Lightning
      </span>
    </div>
  )
}
