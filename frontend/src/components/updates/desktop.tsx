import { useRead } from "@hooks";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuTrigger,
} from "@ui/dropdown";
import { Bell } from "lucide-react";
import { SingleUpdate } from "./update";
import { Button } from "@ui/button";

export const DesktopUpdates = () => {
  const updates = useRead("ListUpdates", {}).data;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger>
        <Button variant="ghost">
          <Bell className="w-4 h-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-[500px]">
        <DropdownMenuGroup>
          {updates?.map((update) => (
            <SingleUpdate update={update} />
          ))}
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
};
