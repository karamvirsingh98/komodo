import { useRead, useWrite } from "@lib/hooks";
import { Types } from "@monitor/client";
import { useState } from "react";
import {
  AccountSelector,
  ConfigInput,
  ConfigItem,
} from "@components/config/util";
import { ImageConfig } from "./components/image";
import { RestartModeSelector } from "./components/restart";
import { NetworkModeSelector } from "./components/network";
import { PortsConfig } from "./components/ports";
import { EnvVars } from "./components/environment";
import { VolumesConfig } from "./components/volumes";
import { ExtraArgs } from "./components/extra-args";
import { Config } from "@components/config";
import {
  DefaultTerminationSignal,
  TermSignalLabels,
  TerminationTimeout,
} from "./components/term-signal";
import { LabelsConfig, ResourceSelector } from "@components/resources/common";

export const ServerSelector = ({
  selected,
  set,
}: {
  selected: string | undefined;
  set: (input: Partial<Types.DeploymentConfig>) => void;
}) => (
  <ConfigItem label="Server">
    <ResourceSelector
      type="Server"
      selected={selected}
      onSelect={(server_id) => set({ server_id })}
    />
  </ConfigItem>
);

export const DeploymentConfig = ({ id }: { id: string }) => {
  const perms = useRead("GetPermissionLevel", {
    target: { type: "Deployment", id },
  }).data;
  const config = useRead("GetDeployment", { deployment: id }).data?.config;
  const [update, set] = useState<Partial<Types.DeploymentConfig>>({});
  const { mutate } = useWrite("UpdateDeployment");

  if (!config) return null;

  const show_ports = update.network
    ? update.network !== "host"
    : config.network
    ? config.network !== "host"
    : false;

  const disabled = perms !== Types.PermissionLevel.Write;

  return (
    <Config
      disabled={disabled}
      config={config}
      update={update}
      set={set}
      onSave={() => mutate({ id, config: update })}
      components={{
        general: {
          "": {
            server_id: (value, set) => (
              <ServerSelector selected={value} set={set} />
            ),
          },
          container: {
            image: (value, set) => (
              <ImageConfig image={value} set={set} disabled={disabled} />
            ),
            docker_account: (value, set) => (
              <AccountSelector
                id={update.server_id ?? config.server_id}
                account_type="docker"
                type="Server"
                selected={value}
                onSelect={(docker_account) => set({ docker_account })}
                disabled={disabled}
              />
            ),
            restart: (value, set) => (
              <RestartModeSelector
                selected={value}
                set={set}
                disabled={disabled}
              />
            ),
            process_args: (value, set) => (
              <ConfigInput
                label="Process Args"
                value={value}
                onChange={(process_args) => set({ process_args })}
                disabled={disabled}
              />
            ),
            network: (value, set) => (
              <NetworkModeSelector
                server_id={update.server_id ?? config.server_id}
                selected={value}
                onSelect={(network) => set({ network })}
                disabled={disabled}
              />
            ),
            ports: (value, set) =>
              show_ports && <PortsConfig ports={value ?? []} set={set} />,
            volumes: (v, set) => <VolumesConfig volumes={v ?? []} set={set} />,
            labels: (l, set) => <LabelsConfig labels={l ?? []} set={set} />,
            extra_args: (value, set) => (
              <ExtraArgs args={value ?? []} set={set} disabled={disabled} />
            ),
          },
          settings: {
            send_alerts: true,
            redeploy_on_build:
              (update.image?.type || config.image?.type) === "Build",
          },
        },
        environment: {
          environment: {
            environment: (vars, set) => (
              <EnvVars
                vars={vars ?? []}
                set={set}
                server={update.server_id || config.server_id}
                disabled={disabled}
              />
            ),
            skip_secret_interp: true,
          },
        },
        termination: {
          termination: {
            termination_signal: (value, set) => (
              <DefaultTerminationSignal
                arg={value}
                set={set}
                disabled={disabled}
              />
            ),
            termination_timeout: (value, set) => (
              <TerminationTimeout arg={value} set={set} disabled={disabled} />
            ),
            term_signal_labels: (value, set) => (
              <TermSignalLabels args={value} set={set} disabled={disabled} />
            ),
          },
        },
      }}
    />
  );
};
