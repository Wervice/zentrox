function _layout($$payload, $$props) {
  let { children } = $$props;
  children($$payload);
  $$payload.out += `<!---->`;
}
export {
  _layout as default
};
