// Align an opject to the middle of the x axis.

function XAlign({ className, children }) {
 return (
  <span className={"inline-flex items-center " + className}>{children}</span>
 );
}

export { XAlign };
