"use client";

import dynamic from "next/dynamic";
import { useEffect, useRef, useState } from "react";

const Dithering = dynamic(
  () => import("@paper-design/shaders-react").then((mod) => mod.Dithering),
  { ssr: false },
);

const GrainGradient = dynamic(
  () => import("@paper-design/shaders-react").then((mod) => mod.GrainGradient),
  { ssr: false },
);

/** Shared intersection observer — same pattern as Fumadocs */
let observer: IntersectionObserver | undefined;
const observerTargets = new WeakMap<
  Element,
  (e: IntersectionObserverEntry) => void
>();

function useIsVisible(ref: React.RefObject<HTMLElement | null>) {
  const [visible, setVisible] = useState(false);
  useEffect(() => {
    observer ??= new IntersectionObserver((entries) => {
      for (const entry of entries) observerTargets.get(entry.target)?.(entry);
    });
    const el = ref.current;
    if (!el) return;
    observerTargets.set(el, (entry) => setVisible(entry.isIntersecting));
    observer.observe(el);
    return () => {
      observer?.unobserve(el);
      observerTargets.delete(el);
    };
  }, [ref]);
  return visible;
}

/**
 * Dithering warp canvas — the halftone dot-wave effect from fumadocs.dev
 * Adapts color to light/dark mode.
 */
export function HeroCanvas({ className }: { className?: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const visible = useIsVisible(ref);

  return (
    <div ref={ref} className={className} aria-hidden="true">
      <Dithering
        colorBack="#00000000"
        colorFront="#beb5b1"
        shape="warp"
        type="4x4"
        speed={visible ? 0.3 : 0}
        className="opacity-30 dark:opacity-20 w-full h-full"
        minPixelRatio={1}
      />
    </div>
  );
}

/**
 * GrainGradient — the smoky colour-cloud background from fumadocs hero.
 * Used on the left sticky panel.
 */
export function HeroGradient({ className }: { className?: string }) {
  const ref = useRef<HTMLDivElement>(null);
  const visible = useIsVisible(ref);
  const [ready, setReady] = useState(false);

  useEffect(() => {
    const t = setTimeout(() => setReady(true), 300);
    return () => clearTimeout(t);
  }, []);

  return (
    <div ref={ref} className={className} aria-hidden="true">
      {ready && (
        <GrainGradient
          className="w-full h-full"
          colors={["#1a1a1a", "#2a1a0a", "#0a0a0a"]}
          colorBack="#00000000"
          softness={1}
          intensity={0.6}
          noise={0.45}
          speed={visible ? 0.4 : 0}
          shape="corners"
          minPixelRatio={1}
        />
      )}
    </div>
  );
}
