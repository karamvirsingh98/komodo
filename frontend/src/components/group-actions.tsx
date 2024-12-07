import {
  useSelectedResources,
  useExecute,
  useWrite,
  useUser,
  useRead,
} from "@lib/hooks";
import { UsableResource } from "@types";
import { Button } from "@ui/button";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@ui/dialog";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@ui/dropdown-menu";
import { Input } from "@ui/input";
import { Types } from "komodo_client";
import { ChevronDown, CheckCircle } from "lucide-react";
import { useState } from "react";
import { ConfirmButton } from "./util";
import { useToast } from "@ui/use-toast";
import { usableResourceExecuteKey } from "@lib/utils";

export const GroupActions = <
  T extends Types.ExecuteRequest["type"] | Types.WriteRequest["type"],
>({
  type,
  actions,
}: {
  type: UsableResource;
  actions: T[];
}) => {
  const [action, setAction] = useState<T>();
  const [selected] = useSelectedResources(type);

  return (
    <>
      <GroupActionDropdownMenu
        type={type}
        actions={actions}
        onSelect={setAction}
        disabled={!selected.length}
      />
      <GroupActionDialog
        type={type}
        action={action}
        onClose={() => setAction(undefined)}
      />
    </>
  );
};

const GroupActionDropdownMenu = <
  T extends Types.ExecuteRequest["type"] | Types.WriteRequest["type"],
>({
  type,
  actions,
  onSelect,
  disabled,
}: {
  type: UsableResource;
  actions: T[];
  onSelect: (item: T) => void;
  disabled: boolean;
}) => {
  const [selected] = useSelectedResources(type);

  const admin = useUser().data?.admin;
  const permissions = useRead("ListPermissions", {}).data;
  const resources = useRead(`List${type}s`, {}).data;

  const can_write =
    admin ||
    selected.every((name) => {
      const rx = resources?.find((rx) => rx.name === name);
      const pm = permissions?.find((p) => p.resource_target.id === rx?.id);
      if (pm?.level === Types.PermissionLevel.Write) {
        return true;
      } else {
        return false;
      }
    });

  const can_execute =
    admin ||
    selected.every((name) => {
      const rx = resources?.find((rx) => rx.name === name);
      const pm = permissions?.find((p) => p.resource_target.id === rx?.id);
      if (
        pm?.level === Types.PermissionLevel.Write ||
        pm?.level === Types.PermissionLevel.Execute
      ) {
        return true;
      } else {
        return false;
      }
    });

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild disabled={disabled}>
        <Button variant="outline" className="w-40 justify-between">
          Execute
          <ChevronDown className="w-4" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent
        align="start"
        className={type === "Server" ? "w-56" : "w-40"}
      >
        {type === "ResourceSync" && (
          <DropdownMenuItem
            onClick={() => onSelect("RefreshResourceSyncPending" as any)}
            className="focus:bg-secondary"
          >
            <Button variant="secondary" size="sm" className="w-full">
              Refresh
            </Button>
          </DropdownMenuItem>
        )}
        {can_execute &&
          actions.map((action) => (
            <DropdownMenuItem
              key={action}
              onClick={() => onSelect(action)}
              className="focus:bg-secondary"
            >
              <Button variant="secondary" size="sm" className="w-full ">
                {action
                  .replaceAll("Batch", "")
                  .replaceAll(type, "")
                  .match(/[A-Z][a-z]+/g)
                  ?.join(" ")}
              </Button>
            </DropdownMenuItem>
          ))}
        {can_write && (
          <DropdownMenuItem
            onClick={() => onSelect(`Delete${type}` as any)}
            className="focus:bg-destructive"
          >
            <Button variant="destructive" size="sm" className="w-full">
              Delete
            </Button>
          </DropdownMenuItem>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
};

const GroupActionDialog = ({
  type,
  action,
  onClose,
}: {
  type: UsableResource;
  action:
    | (Types.ExecuteRequest["type"] | Types.WriteRequest["type"])
    | undefined;
  onClose: () => void;
}) => {
  const { toast } = useToast();
  const [selected, setSelected] = useSelectedResources(type);
  const [text, setText] = useState("");

  const { mutate: execute, isPending: executePending } = useExecute(
    action! as Types.ExecuteRequest["type"],
    { onSuccess: onClose }
  );
  const { mutate: write, isPending: writePending } = useWrite(
    action! as Types.WriteRequest["type"],
    {
      onSuccess: () => {
        if (action?.includes("Delete")) setSelected([]);
        onClose();
      },
    }
  );

  if (!action) return;

  const formatted = action.replaceAll("Batch", "").replaceAll(type, "");
  const isPending = executePending || writePending;

  return (
    <Dialog open={!!action} onOpenChange={(o) => !o && onClose()}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Execute Action - {formatted}</DialogTitle>
        </DialogHeader>
        <div className="py-8 flex flex-col gap-4">
          <ul className="mb-8 p-4 bg-accent text-sm list-disc list-inside rounded-md">
            {selected.map((resource) => (
              <li key={resource}>{resource}</li>
            ))}
          </ul>
          {!action.startsWith("Refresh") && (
            <>
              <p
                onClick={() => {
                  navigator.clipboard.writeText(formatted);
                  toast({ title: `Copied "${formatted}" to clipboard!` });
                }}
                className="cursor-pointer"
              >
                Please enter <b>{formatted}</b> below to confirm this action.
                <br />
                <span className="text-xs text-muted-foreground">
                  You may click the action in bold to copy it
                </span>
              </p>
              <Input value={text} onChange={(e) => setText(e.target.value)} />
            </>
          )}
        </div>
        <DialogFooter>
          <ConfirmButton
            title="Confirm"
            icon={<CheckCircle className="w-4 h-4" />}
            onClick={() => {
              for (const resource of selected) {
                if (action.startsWith("Delete")) {
                  write({ id: resource } as any);
                } else if (action.startsWith("Refresh")) {
                  write({ [usableResourceExecuteKey(type)]: resource } as any);
                } else {
                  execute({
                    [usableResourceExecuteKey(type)]: resource,
                  } as any);
                }
              }
              if (action.startsWith("Delete")) {
                setSelected([]);
              }
            }}
            disabled={action.startsWith("Refresh") ? false : text !== formatted}
            loading={isPending}
          />
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
