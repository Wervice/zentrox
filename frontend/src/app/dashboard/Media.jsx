import { useState, useEffect } from "react";
import Page from "@/components/ui/PageWrapper";
import { fetchURLPrefix } from "@/lib/fetchPrefix";
import { Toaster } from "@/components/ui/toaster";
import { Button } from "@/components/ui/button";
import { Link } from "lucide-react";
import { DataTable } from "@/components/ui/dataTable";
import { toast } from "@/components/ui/use-toast";
import { Switch } from "@/components/ui/switch";

function Media() {
  const [mediaEnabled, setMediaEnabled] = useState(false);
  const [locations, setLocations] = useState();
  const [fetchedLocations, setFetchedLocations] = useState(false);

  useEffect(() => {
    if (!fetchedLocations) {
      fetch(`${fetchURLPrefix}/api/getVideoSourceList`).then((res) => {
        if (res.ok) {
          res.json().then((j) => setLocations(j.locations));
        } else {
          toast({
            title: "Failed to fetch video location list",
            description: "Zentrox failed to fetch the list of video locations",
          });
        }
      });
      setFetchedLocations(true);
    }
  }, [fetchedLocations]);

  useEffect(() => {
    fetch(fetchURLPrefix + "/api/getEnableMedia").then((r) => {
      if (r.ok) {
        r.json().then((j) => {
          setMediaEnabled(j.enabled);
        });
      } else {
        toast({ title: "Failed to fetch media status" });
      }
    });
  }, [mediaEnabled]);

  const updateList = () => {
    fetch(`${fetchURLPrefix}/api/updateVideoSourceList`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        locations,
      }),
    });
  };

  return (
    <Page name="Media">
      <Toaster />
      <div className="flex items-center">
        <Switch
          onCheckedChange={(e) => {
            fetch(`${fetchURLPrefix}/api/setEnableMedia/` + e).then((res) => {
              if (!res.ok) {
                toast({
                  title: "Failed to change media status",
                  description: e
                    ? "Failed to enable media center"
                    : "Failed to disable media center",
                });
              }
            });
            setMediaEnabled(e);
          }}
          checked={mediaEnabled}
          className="inline-block mr-1 align-middle"
          id={"cb"}
        />{" "}
        <label htmlFor={"cb"}>Enabled</label>
      </div>
      <span className="text-sm">
        Zentrox Media Center can display video and music.
      </span>
      <br />{" "}
      <Button
        variant="link"
        className="p-0 m-0"
        onClick={() => {
          window.open("/media");
        }}
      >
        <Link className="w-4 h-4 mr-1" /> Open Media Center
      </Button>
      <strong className="block mt-1">Sources</strong>
      <span className="text-sm block mb-1">
        Add media source for display in media center.
      </span>
      <DataTable
        enabled={mediaEnabled}
        entries={locations}
        onEntriesChange={setLocations}
      >
        <Button className="ml-1" onClick={() => updateList()}>
          Apply changes
        </Button>
      </DataTable>
    </Page>
  );
}

export default Media;
