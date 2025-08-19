let _anime: any | null = null;
function getAnime(): Promise<any> {
  if (_anime) return Promise.resolve(_anime);
  return import('animejs/lib/anime.es.js').then((m) => {
    _anime = (m as any).default ?? m;
    return _anime;
  });
}

// Fade-in on mount
export type FadeInParams = {
  duration?: number;
  delay?: number;
  y?: number; // translateY from -> 0
  opacityFrom?: number;
  easing?: string;
};

export function fadeIn(node: HTMLElement, params: FadeInParams = {}) {
  const cfg = {
    duration: 600,
    delay: 0,
    y: 8,
    opacityFrom: 0,
    easing: 'easeOutQuad',
    ...params,
  };

  // Set initial state and animate in
  node.style.opacity = String(cfg.opacityFrom);
  node.style.transform = `translateY(${cfg.y}px)`;
  const play = () => {
    getAnime().then((anime) => {
      anime.remove(node);
      anime({
        targets: node,
        opacity: [cfg.opacityFrom, 1],
        translateY: [cfg.y, 0],
        duration: cfg.duration,
        delay: cfg.delay,
        easing: cfg.easing,
      });
    });
  };

  play();

  return {
    update(newParams: FadeInParams = {}) {
      Object.assign(cfg, newParams);
    },
    destroy() {
      getAnime().then((anime) => anime.remove(node)).catch(() => {});
      node.style.removeProperty('opacity');
      node.style.removeProperty('transform');
    },
  };
}

// Hover lift micro-interaction
export type HoverLiftParams = {
  y?: number; // pixels to lift
  scale?: number;
  duration?: number;
};

export function hoverLift(node: HTMLElement, params: HoverLiftParams = {}) {
  const cfg = { y: 2, scale: 1, duration: 180, ...params };

  function onEnter() {
    getAnime().then((anime) => {
      anime.remove(node);
      anime({
        targets: node,
        translateY: -cfg.y!,
        scale: cfg.scale,
        duration: cfg.duration,
        easing: 'easeOutQuad',
      });
    });
  }

  function onLeave() {
    getAnime().then((anime) => {
      anime.remove(node);
      anime({
        targets: node,
        translateY: 0,
        scale: 1,
        duration: Math.min(200, cfg.duration || 180),
        easing: 'easeOutQuad',
      });
    });
  }

  node.addEventListener('mouseenter', onEnter);
  node.addEventListener('mouseleave', onLeave);

  return {
    update(newParams: HoverLiftParams = {}) {
      Object.assign(cfg, newParams);
    },
    destroy() {
      getAnime().then((anime) => anime.remove(node)).catch(() => {});
      node.removeEventListener('mouseenter', onEnter);
      node.removeEventListener('mouseleave', onLeave);
    },
  };
}

// Click ripple effect
export type RippleParams = {
  duration?: number;
  opacity?: number; // initial opacity
  centered?: boolean;
  color?: string; // optional override color
};

export function ripple(node: HTMLElement, params: RippleParams = {}) {
  const cfg = { duration: 550, opacity: 0.28, centered: false, ...params };

  function onPointerDown(e: PointerEvent | MouseEvent) {
    // Respect disabled/loading states on buttons
    if ((node as HTMLButtonElement).disabled || node.getAttribute('aria-busy') === 'true') return;

    const rect = node.getBoundingClientRect();
    const x = cfg.centered
      ? rect.width / 2
      : 'clientX' in e
        ? (e.clientX - rect.left)
        : rect.width / 2;
    const y = cfg.centered
      ? rect.height / 2
      : 'clientY' in e
        ? (e.clientY - rect.top)
        : rect.height / 2;

    const maxDim = Math.max(rect.width, rect.height);
    const radius = maxDim * 0.75;

    const el = document.createElement('span');
    el.className = 'ripple';
    if (cfg.color) el.style.background = cfg.color;

    el.style.left = `${x}px`;
    el.style.top = `${y}px`;
    el.style.width = `${radius * 2}px`;
    el.style.height = `${radius * 2}px`;

    node.appendChild(el);

    getAnime().then((anime) => {
      anime({
        targets: el,
        scale: [0, 1],
        opacity: [cfg.opacity, 0],
        duration: cfg.duration,
        easing: 'easeOutQuad',
        complete: () => el.remove(),
      });
    });
  }

  node.addEventListener('pointerdown', onPointerDown);

  return {
    update(newParams: RippleParams = {}) {
      Object.assign(cfg, newParams);
    },
    destroy() {
      node.removeEventListener('pointerdown', onPointerDown);
    },
  };
}
