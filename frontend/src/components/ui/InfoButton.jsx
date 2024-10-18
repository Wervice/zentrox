import { InfoIcon } from "lucide-react";

import {
 Dialog,
 DialogClose,
 DialogContent,
 DialogTitle,
 DialogTrigger,
 DialogFooter,
 DialogHeader,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

export default function InfoButton({ title, info }) {
 return (
  <Dialog>
   <DialogTrigger asChild className="inline-block">
    <InfoIcon className="h-4 w-4 inline-block text-neutral-600" />
   </DialogTrigger>
   <DialogContent>
    <DialogHeader>
     <DialogTitle>{title}</DialogTitle>
    </DialogHeader>
    {info}
    <DialogFooter>
     <DialogClose asChild>
      <Button variant="secondary">Close</Button>
     </DialogClose>
    </DialogFooter>
   </DialogContent>
  </Dialog>
 );
}
