function Placeholder({ children, ...props }) {
  return (
    <div className="w-full p-20" {...props}>
      {children}
    </div>
  );
}

function PlaceholderIcon({ icon }) {
  const Icon = icon;

  return <>{<Icon className="w-full h-16 opacity-75 block mb-4" />}</>;
}

function PlaceholderSubtitle({ children }) {
  return (
    <>
      <span className="block text-lg font-semibold w-full text-center opacity-75">
        {children}
      </span>
    </>
  );
}

export { PlaceholderSubtitle, Placeholder, PlaceholderIcon };
