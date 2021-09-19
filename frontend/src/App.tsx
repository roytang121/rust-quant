import { Navbar, NavbarGroup, NavbarHeading } from "@blueprintjs/core";
import { FC } from "react";
import Lambda from "./features/lambda/Lambda";

const App: FC = () => {
  return (
    <div
      className="bp3 bp3-dark ag-theme-balham-dark"
      style={{ display: "flex", flexDirection: "column", height: "100%" }}
    >
      <Navbar>
        <NavbarGroup>
          <NavbarHeading>Rust Quant</NavbarHeading>
        </NavbarGroup>
      </Navbar>
      <Lambda host={"http://localhost:6008/"}/>
    </div>
  );
};

export default App;
