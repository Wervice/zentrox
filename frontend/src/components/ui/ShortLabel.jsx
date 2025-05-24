export default function Label({ children }) {
  return (
    <label className="font-200 mb-1 text-base flex items-center gap-1">
      {children}
    </label>
  );
}
