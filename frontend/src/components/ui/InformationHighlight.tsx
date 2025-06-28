import { ReactNode, useState } from "react";
import { Button } from "./button";

type IconProps = {
  className: String;
};

function InformationHighlight({
  title,
  children,
  Icon,
  collapsible = false,
}: {
  title: String;
  children: ReactNode;
  Icon: React.FC<IconProps>;
  collapsible: boolean;
}) {
  const [isOpen, setIsOpen] = useState(!collapsible);

  return (
    <div className="p-2 rounded border border-neutral-900 mb-2">
      <span className="w-full flex items-center opacity-75">
        <Icon className="h-4 w-4 mr-1" />
        {title}
      </span>
      {isOpen && <span className="text-xl">{children}</span>}
      {collapsible && (
        <Button
          className="block mt-2"
          variant="secondary"
          onClick={() => setIsOpen(!isOpen)}
        >
          {isOpen ? "Collaps" : "Expand"}
        </Button>
      )}
    </div>
  );
}

export default InformationHighlight;
