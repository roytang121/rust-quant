import {
  Button,
  Menu,
  MenuItem,
  Navbar,
  NavbarGroup,
  NavbarHeading,
  NonIdealState,
  Popover,
} from "@blueprintjs/core";
import { isEmpty, isNil } from "lodash";
import { FC, useState } from "react";
import { EnvConfig, staticEnvConfig, Env } from "./app/envConfig";
import Lambda from "./features/lambda/Lambda";

const App: FC = () => {
  const [envConfig, setEnvConfig] = useState<EnvConfig>();
  const [instance, setInstance] = useState<string>();

  const label = envConfig ? `${envConfig?.name} - ${instance}` : `Select Env`;

  return (
    <div
      className="bp3 bp3-dark ag-theme-balham-dark"
      style={{ display: "flex", flexDirection: "column", height: "100%" }}
    >
      <Navbar>
        <NavbarGroup>
          <NavbarHeading>Rust Quant</NavbarHeading>
        </NavbarGroup>
        <NavbarGroup align="right">
          <Popover>
            <Button text={label} intent="none" />
            <Menu>
              {staticEnvConfig.map((config) => (
                <MenuItem key={config.name} text={config.name}>
                  {config.instances.map((instance) => (
                    <MenuItem
                      key={instance}
                      text={instance}
                      onClick={() => {
                        setEnvConfig(config);
                        setInstance(instance);
                      }}
                    />
                  ))}
                </MenuItem>
              ))}
            </Menu>
          </Popover>
        </NavbarGroup>
      </Navbar>
      {(isNil(envConfig) || isEmpty(instance)) && (
        <NonIdealState title="Select Lambda Instance" icon="console" />
      )}
      {envConfig && instance && (
        <Lambda host={envConfig.grpc_host} instance={instance} />
      )}
    </div>
  );
};

export default App;
