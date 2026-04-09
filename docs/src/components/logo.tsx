/** biome-ignore-all lint/performance/noImgElement: <explanation> */
export function Logo({ className }: { className?: string }) {
  return (
    <div className="space-x-2 flex flex-row items-center justify-center">
      <img
        className={`${className} h-4.5 w-auto`}
        src="/favicon.svg"
        alt="favicon"
      />
      <span className="text-lg font-monospace">RS-AUTH</span>
    </div>
  );
}
