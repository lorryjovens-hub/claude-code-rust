import { createElement } from 'react';
import { useSSR } from './useSSR.js';
import { composeInitialProps } from './context.js';
import { getDisplayName } from './utils.js';
export const withSSR = () => function Extend(WrappedComponent) {
  function I18nextWithSSR(_ref) {
    let {
      initialI18nStore,
      initialLanguage,
      ...rest
    } = _ref;
    useSSR(initialI18nStore, initialLanguage);
    return createElement(WrappedComponent, {
      ...rest
    });
  }
  I18nextWithSSR.getInitialProps = composeInitialProps(WrappedComponent);
  I18nextWithSSR.displayName = `withI18nextSSR(${getDisplayName(WrappedComponent)})`;
  I18nextWithSSR.WrappedComponent = WrappedComponent;
  return I18nextWithSSR;
};