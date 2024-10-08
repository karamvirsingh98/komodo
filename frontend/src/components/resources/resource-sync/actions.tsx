import { ActionButton, ActionWithDialog } from "@components/util";
import { useExecute, useInvalidate, useRead, useWrite } from "@lib/hooks";
import { sync_no_changes } from "@lib/utils";
import { RefreshCcw, SquarePlay } from "lucide-react";

export const RefreshSync = ({ id }: { id: string }) => {
  const inv = useInvalidate();
  const { mutate, isPending } = useWrite("RefreshResourceSyncPending", {
    onSuccess: () => inv(["GetResourceSync", { sync: id }]),
  });
  const pending = isPending;
  return (
    <ActionButton
      title="Refresh"
      icon={<RefreshCcw className="w-4 h-4" />}
      onClick={() => mutate({ sync: id })}
      disabled={pending}
      loading={pending}
    />
  );
};

export const ExecuteSync = ({ id }: { id: string }) => {
  const { mutate, isPending } = useExecute("RunSync");
  const syncing = useRead(
    "GetResourceSyncActionState",
    { sync: id },
    { refetchInterval: 5000 }
  ).data?.syncing;
  const sync = useRead("GetResourceSync", { sync: id }).data;

  if (!sync || sync_no_changes(sync)) return null;

  const pending = isPending || syncing;

  return (
    <ActionWithDialog
      name={sync.name}
      title="Execute Sync"
      icon={<SquarePlay className="w-4 h-4" />}
      onClick={() => mutate({ sync: id })}
      disabled={pending}
      loading={pending}
    />
  );
};
