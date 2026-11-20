import { computed, type ComputedRef } from "vue";
import { useMediaQuery } from "@vueuse/core";
import { isMobileDevice } from "@/lib/utils";

/**
 * A phone or tablet, including when the user agent looks like a desktop
 * (iPadOS "Request Desktop Site", browser device emulation, ...).
 *
 * The width check is paired with a coarse pointer so a narrow desktop window
 * still keeps its title bar (and therefore its window controls).
 */
export function useIsMobileDevice(): ComputedRef<boolean> {
  const isNarrowTouchDevice = useMediaQuery(
    "(max-width: 767px) and (pointer: coarse)",
  );

  return computed(() => isMobileDevice() || isNarrowTouchDevice.value);
}
