import "./styles.css";

import { setupAccessForm } from "./access";
import {
  configureProductLinks,
  enableProductFallbackFocus,
  PRODUCT_FALLBACK,
} from "./landing";
import { setupMotion } from "./motion";

const productHref = configureProductLinks(document, import.meta.env.VITE_PRODUCT_URL);
if (productHref === PRODUCT_FALLBACK) {
  enableProductFallbackFocus(document);
}

setupAccessForm(document, import.meta.env.VITE_ACCESS_REQUEST_URL, window.fetch.bind(window));
setupMotion(document, window);
