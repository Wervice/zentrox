import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { Button } from "@/components/ui/button";

function DataTable({
  entries = [], // Set a default structure for entries
  onEntriesChange = function (_i) {},
  className,
  checkboxTitle = "Enabled",
  children = <></>,
  key,
}) {
  const deleteEntry = (index) => {
    onEntriesChange(
      entries.filter((_e, i) => i !== index), // Return only entries that don't match the index
    );
  };

  const handleInputChange = (index, key, value) => {
    // Update the entries based on index and key (0 or 1)
    const updatedEntries = entries.map((entry, i) =>
      i === index
        ? [key === 0 ? value : entry[0], key === 1 ? value : entry[1], entry[2]]
        : entry,
    );
    onEntriesChange(updatedEntries);
  };

  const handleCheckboxChange = (index, checked) => {
    // Update the checkbox status
    const updatedEntries = entries.map((entry, i) =>
      i === index ? [entry[0], entry[1], checked] : entry,
    );
    onEntriesChange(updatedEntries);
  };

  return (
    <span key={key} className={className}>
      <Button
        variant="secondary"
        className="mb-1 mt-1"
        onClick={() => {
          onEntriesChange(entries.concat([["", "", true]]));
        }}
      >
        Add source
      </Button>
      {children}
      {entries.length === 0 ? (
        <div className="p-2 text-sm text-white/80">
          Add new sources to media center
        </div>
      ) : (
        <></>
      )}
      {entries.map((element, index) => {
        const [text1 = "", text2 = "", isChecked = false] = element;

        return (
          <>
            <span key={index} className="block mb-1">
              <Checkbox
                className="inline-block mr-1"
                checked={isChecked}
                title={checkboxTitle}
                onCheckedChange={(checked) =>
                  handleCheckboxChange(index, checked)
                }
              />
              <Input
                type="text"
                className="inline-block mr-1"
                value={text1}
                disabled={!isChecked}
                onChange={(e) => handleInputChange(index, 0, e.target.value)}
                placeholder="Source path"
              />
              <Input
                type="text"
                className="inline-block mr-2"
                value={text2}
                disabled={!isChecked}
                onChange={(e) => handleInputChange(index, 1, e.target.value)}
                placeholder="Source alias"
              />
              <Button
                variant="outline"
                className="inline-block border-red-500 text-red-500 hover:bg-red-500/5 hover:text-red-500"
                onClick={() => deleteEntry(index)}
              >
                Delete
              </Button>
            </span>
          </>
        );
      })}
    </span>
  );
}

export { DataTable };
