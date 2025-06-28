import { useState, useRef, useEffect } from "react";
import Page from "@/components/ui/PageWrapper";
import { Button } from "@/components/ui/button";
import useNotification from "@/lib/notificationState";
import { fetchURLPrefix } from "@/lib/fetchPrefix";

function Server() {
  var tlsCertFileInput = useRef();
  const { deleteNotification, notify, notifications } = useNotification();
  const [certNames, setCertNames] = useState("");
  const fetchData = () => {
    fetch(fetchURLPrefix + "/api/certNames").then((res) => {
      res.json().then((j) => {
        setCertNames(j);
      });
    });
  };

  useEffect(fetchData, []);

  return (
    <Page name="Server">
      <h2 className="font-semibold">TLS</h2>
      <input
        type="file"
        ref={tlsCertFileInput}
        hidden={true}
        onChange={() => {
          var fileForSubmit = tlsCertFileInput.current.files[0];
          if (fileForSubmit.size >= 1024 * 1024 * 1024 * 1) {
            notify("The file you provided was larger than 1GB");
          }

          var fileName = tlsCertFileInput.current.files[0].name;

          if (fileName.split(".").reverse()[0].toLowerCase() != "pem") {
            notify("Zentrox can only use pem certificates.");
            return;
          }

          var formData = new FormData();
          formData.append("file", fileForSubmit);
          fetch(fetchURLPrefix + "/upload/tls", {
            method: "POST",
            body: formData,
          }).then((res) => {
            if (res.ok) {
              setCertNames({
                tls: fileName,
              });
              tlsCertFileInput.current.value = "";
              notify(
                "Zentrox successfully uploaded the new certificate. You need to manually restart Zentrox to start using the new certificate.",
              );
            } else {
              notify(
                "Zentrox failed to upload the TLS certificate you provided",
              );
            }
          });
        }}
      />
      <Button
        onClick={() => {
          tlsCertFileInput.current.click();
        }}
        className="mr-1"
      >
        Upload
      </Button>{" "}
      <span className="text-neutral-600">{certNames.tls}</span>
    </Page>
  );
}

export default Server;
