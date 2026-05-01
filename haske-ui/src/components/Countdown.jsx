import { useEffect, useState } from "react"

function formatTime(seconds) {
  const safeSeconds = Math.max(0, seconds)
  const mins = Math.floor(safeSeconds / 60)
  const secs = safeSeconds % 60
  return `${mins}:${secs.toString().padStart(2, "0")}`
}

export default function Countdown({ expiresAt, prefix }) {
  const [now, setNow] = useState(() => Math.floor(Date.now() / 1000))

  useEffect(() => {
    const interval = window.setInterval(() => {
      setNow(Math.floor(Date.now() / 1000))
    }, 1000)
    return () => window.clearInterval(interval)
  }, [])

  const time = expiresAt ? Math.max(0, expiresAt - now) : 0

  const isDanger = time <= 60

  return (
    <div className="rounded-2xl border border-white/10 bg-white/[0.03] px-4 py-3.5">
      <div className="text-xs font-bold uppercase tracking-[0.16em] text-slate-500">
        {prefix}
      </div>
      <div
        className={`mt-1 text-[1.75rem] font-extrabold tracking-[0.12em] sm:text-2xl ${
          isDanger ? "text-[#FF4D4D]" : "text-white"
        }`}
      >
        {formatTime(time)}
      </div>
    </div>
  )
}
