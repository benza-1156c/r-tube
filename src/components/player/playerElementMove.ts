/**
 * Make a state-preserving move of the player a real move, not a reconnect.
 *
 * Video.js elements are built once and never expect to be reconnected:
 * `connectedCallback` creates a new controller (popover, delay timers, popup
 * group member, ...) and the previous one is never released. Moving the player
 * element between the watch page and the mini player reconnects every element
 * in the skin, so each move leaves an extra popover controller behind — and
 * the leftovers fight over the container's popup group. That is why hovering
 * the volume button opened the slider and immediately closed it again.
 *
 * `Element.moveBefore()` keeps element state (no disconnect/connect) only for
 * elements that define `connectedMoveCallback`; without it the browser runs
 * the normal lifecycle. The callback has to exist when the element is defined,
 * hence this wrapper around `customElements.define`. It has to be imported
 * before the Video.js element modules.
 */
const define = customElements.define.bind(customElements);

type CustomElementClass = CustomElementConstructor & { prototype?: object };

customElements.define = function registerElement(
  name: string,
  constructor: CustomElementConstructor,
  options?: ElementDefinitionOptions,
): void {
  const klass = constructor as CustomElementClass;
  const prototype = klass.prototype;
  if (prototype && !("connectedMoveCallback" in prototype)) {
    Object.defineProperty(prototype, "connectedMoveCallback", {
      configurable: true,
      value() {
        // Nothing to do: the element kept its state through the move.
      },
    });
  }
  define(name, constructor, options);
};

export {};
