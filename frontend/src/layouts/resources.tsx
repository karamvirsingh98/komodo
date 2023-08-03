import { NewDeployment } from "@resources/deployment/new";
import { Button } from "@ui/button";
import { Input } from "@ui/input";
import { PlusCircle } from "lucide-react";
import { ReactNode, useState } from "react";
import { Page } from "./page";
import { ResourceTarget } from "@monitor/client/dist/types";
import { NewServer } from "@resources/server/new";
import { NewBuild } from "@resources/build/new";
import { NewBuilder } from "@resources/builder/new";
import { NewAlerter } from "@resources/alerter/new";

export const Resources = ({
  type,
  summary,
  icon,
  components,
}: {
  type: ResourceTarget["type"];
  summary: ReactNode;
  icon: ReactNode;
  components: (search: string) => ReactNode;
}) => {
  const [search, setSearch] = useState("");
  const [open, setOpen] = useState(false);
  return (
    <Page
      title={<h1 className="text-4xl">{type}s</h1>}
      subtitle={
        <h2 className="text-lg text-muted-foreground flex items-center gap-2">
          {icon}
          {summary}
        </h2>
      }
      actions={
        <div className="flex flex-col-reverse md:flex-row gap-4">
          <Input
            className="w-full md:w-[300px]"
            placeholder={`Search ${type}s`}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
          />
          <Button
            className="w-[200px] flex items-center gap-2"
            variant="outline"
            intent="success"
            onClick={() => setOpen(true)}
          >
            <PlusCircle className="w-4 h-4 text-green-500" />
            New {type}
          </Button>
          {type === "Deployment" && <NewDeployment open={open} set={setOpen} />}
          {type === "Server" && <NewServer open={open} set={setOpen} />}
          {type === "Build" && <NewBuild open={open} set={setOpen} />}
          {type === "Builder" && <NewBuilder open={open} set={setOpen} />}
          {type === "Alerter" && <NewAlerter open={open} set={setOpen} />}
        </div>
      }
    >
      <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
        {components(search)}
      </div>
    </Page>
  );
};
