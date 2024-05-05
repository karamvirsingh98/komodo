import { TagsWithBadge } from "@components/tags";
import { Types } from "@monitor/client";
import { DataTable, SortableHeader } from "@ui/data-table";
import { useRead, useTagsFilter } from "@lib/hooks";
import { ResourceLink } from "../common";
import { DeploymentComponents } from ".";
import { HardDrive } from "lucide-react";
import { useCallback } from "react";

export const DeploymentTable = ({
  deployments,
  search,
}: {
  deployments: Types.DeploymentListItem[] | undefined;
  search?: string;
}) => {
  const tags = useTagsFilter();
  const searchSplit = search?.split(" ") || [];
  const servers = useRead("ListServers", {}).data;
  const serverName = useCallback(
    (id: string) => servers?.find((server) => server.id === id)?.name,
    [servers]
  );
  return (
    <DataTable
      tableKey="deployments"
      data={
        deployments?.filter(
          (resource) =>
            tags.every((tag) => resource.tags.includes(tag)) &&
            (searchSplit.length > 0
              ? searchSplit.every((search) => resource.name.includes(search))
              : true)
        ) ?? []
      }
      columns={[
        {
          accessorKey: "name",
          header: ({ column }) => (
            <SortableHeader column={column} title="Name" />
          ),
          cell: ({ row }) => (
            <ResourceLink type="Deployment" id={row.original.id} />
          ),
        },
        {
          accessorKey: "info.image",
          header: ({ column }) => (
            <SortableHeader column={column} title="Image" />
          ),
          cell: ({
            row: {
              original: {
                info: { build_id, image },
              },
            },
          }) => <Image build_id={build_id} image={image} />,
        },
        {
          accessorKey: "info.server_id",
          sortingFn: (a, b) => {
            const sa = serverName(a.original.info.server_id);
            const sb = serverName(b.original.info.server_id);
            
            if (!sa && !sb) return 0;
            if (!sa) return -1;
            if (!sb) return 1;

            if (sa > sb) return 1;
            else if (sa < sb) return -1;
            else return 0
          },
          header: ({ column }) => (
            <SortableHeader column={column} title="Server" />
          ),
          cell: ({ row }) => (
            <ResourceLink type="Server" id={row.original.info.server_id} />
          ),
        },
        {
          accessorKey: "info.state",
          header: ({ column }) => (
            <SortableHeader column={column} title="State" />
          ),
          cell: ({ row }) => (
            <DeploymentComponents.Status.State id={row.original.id} />
          ),
        },
        {
          header: "Tags",
          cell: ({ row }) => {
            return (
              <div className="flex gap-1">
                <TagsWithBadge tag_ids={row.original.tags} />
              </div>
            );
          },
        },
      ]}
    />
  );
};

const Image = ({
  build_id,
  image,
}: {
  build_id: string | undefined;
  image: string;
}) => {
  const builds = useRead("ListBuilds", {}).data;
  if (build_id) {
    const build = builds?.find((build) => build.id === build_id);
    if (build) {
      return <ResourceLink type="Build" id={build_id} />;
    } else {
      return undefined;
    }
  } else {
    const [img] = image.split(":");
    return (
      <div className="flex gap-2 items-center">
        <HardDrive className="w-4 h-4" />
        {img}
      </div>
    );
  }
};