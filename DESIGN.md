# Design System

## Theme

Image-led Okanagan rental brand with deep forest navigation, true white and soft neutral surfaces, gold actions, and restrained coral highlights.

## Color

- Forest: `#1B3A28`
- Deep forest: `#10271B`
- Ink: `#17261C`
- Surface: `#F5F3EE`
- White: `#FFFFFF`
- Gold accent: `#D7A24A`
- Coral highlight: `#EA6A4B`
- Muted text: `#59685E`
- Hairline: `#E7E4DB`

Use the existing CSS variables in `assets/main.css`; do not duplicate token values inside components.

## Typography

- Headings: Newsreader
- Body and controls: Inter
- Small technical labels only: Geist Mono

## Components

- Buttons use 10–11px radii or full pills when already established.
- Form controls use explicit labels, 48px minimum touch height, clear focus rings, and inline errors.
- Cards use the existing 16px radius with a border, not an added wide shadow.

## Layout

Desktop content uses the established 64–72px page gutters. Mobile switches at 860px and uses 18px gutters. Authentication uses a focused two-column photo/form composition on desktop and a single-column form on mobile.

## Motion

Use short opacity/transform transitions only where they clarify state. Respect `prefers-reduced-motion`.
