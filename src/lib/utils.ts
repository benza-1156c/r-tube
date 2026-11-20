export { cn } from "cn";

export const isMobileDevice = (): boolean => {
  if (typeof navigator === "undefined") return false;

  const ua = navigator.userAgent;
  if (/Android|iPhone|iPad|iPod|IEMobile|Opera Mini|Mobile/i.test(ua)) {
    return true;
  }

  // iPadOS 13+ reports itself as a Mac, so fall back to touch support.
  return /Macintosh/i.test(ua) && navigator.maxTouchPoints > 1;
};

/**
 * The user's preferred languages, most preferred first, reduced to base
 * language codes (e.g. ["th-TH", "en-US"] -> ["th", "en"]).
 */
export const getDeviceLanguages = (): string[] => {
  if (typeof navigator === "undefined") return [];

  const preferred = navigator.languages?.length
    ? navigator.languages
    : [navigator.language];

  return [
    ...new Set(
      preferred
        .map((language) => language.split("-")[0]?.toLowerCase())
        .filter((language): language is string => Boolean(language)),
    ),
  ];
};

/** The device's primary language, e.g. "th" for a Thai phone. */
export const getPrimaryDeviceLanguage = (): string =>
  getDeviceLanguages()[0] ?? "en";
