import { useState } from "react";
import { Button } from "./button";
import { ActionTd, ActionTh, Table, Td, Th, Tr } from "./table";
import { ChevronDown, ChevronUp, MinusIcon } from "lucide-react";

export default function VirtualScrollingTable({
  headers = [],
  entries = [],
  steps = 1,
  displayCount = 10,
}) {
  const [currentTop, setCurrentTop] = useState(0);
  const [selectedSortingHeader, setSelecetedSortingHeader] = useState(0);
  const [ascendingDirection, setAscendingDirection] = useState(true);

  function up() {
    if (currentTop - steps < 0) {
      return;
    }

    setCurrentTop(currentTop - steps);
  }

  function down() {
    if (
      currentTop + steps > entries.length - 1 ||
      entries.length - currentTop + steps < displayCount + 2
    ) {
      return;
    }

    setCurrentTop(currentTop + steps);
  }

  return (
    <>
      <Table>
        <Tr>
          {headers.map((h, ind) => {
            return (
              <Th
                onClick={() => {
                  if (ind == selectedSortingHeader) {
                    setAscendingDirection(!ascendingDirection);
                  }
                  setSelecetedSortingHeader(ind);
                }}
                className="cursor-pointer"
              >
                <span className="flex items-center">
                  {ind == selectedSortingHeader ? (
                    ascendingDirection ? (
                      <ChevronDown className="opacity-75 h-4" />
                    ) : (
                      <ChevronUp className="opacity-75 h-4" />
                    )
                  ) : (
                    <MinusIcon className="opacity-75 h-4" />
                  )}
                  {h}
                </span>
              </Th>
            );
          })}
        </Tr>
        {entries
          .toSorted((x, y) => {
            var xUnwraped = x[selectedSortingHeader];
            var yUnwraped = y[selectedSortingHeader];
            if (Array.isArray(xUnwraped)) {
              xUnwraped = xUnwraped[1];
            }
            if (Array.isArray(yUnwraped)) {
              yUnwraped = yUnwraped[1];
            }
            return xUnwraped > yUnwraped == ascendingDirection;
          })
          .slice(currentTop, currentTop + displayCount)
          .map((e) => {
            return (
              <Tr>
                {e.map((ent) => {
                  return (
                    <Td
                      className="max-w-[300px] w-[300px]"
                      onWheel={(e) => {
                        var delta;
                        if (e.wheelDelta) {
                          delta = e.wheelDelta;
                        } else {
                          delta = -1 * e.deltaY;
                        }
                        if (delta < 0) {
                          down();
                        } else if (delta > 0) {
                          up();
                        }
                      }}
                    >
                      {Array.isArray(ent)
                        ? ent[0].length > 20
                          ? ent[0].slice(0, 20) + "..."
                          : ent[0]
                        : ent.length > 20
                          ? ent.slice(0, 20) + "..."
                          : ent}
                    </Td>
                  );
                })}
              </Tr>
            );
          })}
      </Table>
    </>
  );
}
