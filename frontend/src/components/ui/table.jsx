import "../../app/dashboard/scroll.css";

function Table({ children }) {
  return (
    <span className="overflow-hidden border-2 border-b-0 border-neutral-800 block w-fit rounded-sm mt-1 mb-1">
      <table className="border-spacing-0 border-collapse">{children}</table>
    </span>
  );
}

function Tr({ children }) {
  return (
    <tr className={"border-b-2 border-neutral-800 hover:bg-white/10 "}>
      {children}
    </tr>
  );
}

function Td({ children, className, expand }) {
  return (
    <td
      className={
        "p-2 border-0 " +
        (className || "") +
        " " +
        (!expand ? "max-w[250px]" : "")
      }
    >
      <span className="block max-w-full w-full overflow-x-scroll no-scroll">
        {children}
      </span>
    </td>
  );
}

function ActionTd({ children, className }) {
  return (
    <td className={"p-2 border-0 w-min min-w-0 " + (className || "")}>
      {children}
    </td>
  );
}

function Th({ children, expand }) {
  return (
    <Td className={"bg-white/5"} expand={expand}>
      {children}
    </Td>
  );
}

function ActionTh() {
  return <Td className={"bg-white/5 w-min min-w-0"}></Td>;
}

export { Tr, Td, Table, Th, ActionTh, ActionTd };
