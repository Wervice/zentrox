function SideWayBarChart({ percentage }) {
  const barStyle = {
    width: `${2 * Math.round(percentage)}px`,
    height: `30px`,
  };

  return (
    <div
      className="rounded overflow-hidden content-center items-center flex mb-2"
      style={{
        width: "fit-content",
        height: "30px",
      }}
    >
      <div
        className="inline-block bg-blue-700 mr-2 rounded transition-width duration-700 animate-expand-width"
        style={barStyle}
      ></div>
      {Math.round(percentage)}%
    </div>
  );
}

module.exports = { SideWayBarChart };
