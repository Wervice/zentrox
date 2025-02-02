export default function Image({ src = "" }, { ...props } = []) {
  return <img src={src} className="w-20 m-0" {...props} />;
}
