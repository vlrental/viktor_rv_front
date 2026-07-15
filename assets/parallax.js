(() => {
  if (window.__vlParallaxStarted) return;
  window.__vlParallaxStarted = true;

  const reducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)");
  let band = null;
  let image = null;
  let frame = 0;

  const findBand = () => {
    if (band?.isConnected && image?.isConnected) return true;
    band = document.getElementById("home-parallax-cta");
    image = band?.querySelector(".cta-img") || null;
    return Boolean(band && image);
  };

  const update = () => {
    frame = 0;
    if (!findBand()) return;
    if (reducedMotion.matches) {
      image.style.transform = "none";
      return;
    }

    const rect = band.getBoundingClientRect();
    const viewportHeight = window.innerHeight || document.documentElement.clientHeight;
    const bandCenter = rect.top + rect.height / 2;
    const viewportCenter = viewportHeight / 2;
    const offset = Math.max(-112, Math.min(112, (viewportCenter - bandCenter) * 0.26));
    image.style.transform = `translate3d(0, ${offset.toFixed(1)}px, 0) scale(1.06)`;
    image.dataset.parallaxOffset = offset.toFixed(1);
  };

  const schedule = () => {
    if (!frame) frame = requestAnimationFrame(update);
  };

  window.addEventListener("scroll", schedule, { passive: true });
  document.addEventListener("scroll", schedule, { passive: true, capture: true });
  window.addEventListener("resize", schedule, { passive: true });
  reducedMotion.addEventListener?.("change", schedule);

  const observer = new MutationObserver(schedule);
  observer.observe(document.documentElement, { childList: true, subtree: true });
  schedule();
})();
