import "../../app/dashboard/scroll.css";

function Table({ children, className, ...props }) {
  return (
    <span
      className={
        "overflow-hidden border-2 border-neutral-800 block w-fit rounded-sm mt-1 mb-1 " +
        className
      }
      {...props}
    >
      <table className="border-spacing-0 border-collapse">
        <tbody>{children}</tbody>
      </table>
    </span>
  );
}

function Tr({ children, ...props }) {
  return (
    <tr
      className={"border-b-2 border-neutral-800 hover:bg-white/10 "}
      {...props}
    >
      {children}
    </tr>
  );
}

function Td({ children, className, ...props }) {
  return (
    <td className={"p-2 border-0 " + (className || "")} {...props}>
      <span className="block max-w-full w-full overflow-x-scroll no-scroll">
        {children}
      </span>
    </td>
  );
}

function ActionTd({ children, className }) {
  return (
    <td className={"p-2 border-0 w-max min-w-0 " + (className || "")}>
      {children}
    </td>
  );
}

function Th({ children, className, ...props }) {
  return (
    <Td className={"bg-white/5 " + (className || "")} {...props}>
      {children}
    </Td>
  );
}

function ActionTh() {
  return <Td className={"bg-white/5 w-min min-w-0"}></Td>;
}

export { Tr, Td, Table, Th, ActionTh, ActionTd };
