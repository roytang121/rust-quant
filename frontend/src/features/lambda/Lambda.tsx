import { ColDef } from "ag-grid-community";
import { AgGridReact } from "ag-grid-react";
import { memo, useEffect, useMemo, useState } from "react";
import { timer } from "rxjs";
import LambdaApi from "./lambda.api";
import { LamabdaParamEntry } from "./lambda.types";

interface Props {
  host: string;
}

const Lambda = ({ host }: Props) => {
  const [api] = useState(LambdaApi(host));
  const [entries, setEntries] = useState<LamabdaParamEntry[]>([]);
  useEffect(() => {
    const sub = timer(0, 1000).subscribe(async (_) => {
      const response = await api.getStates();
      const entries: LamabdaParamEntry[] = await response.json();
      setEntries(entries);
    });
    return () => sub.unsubscribe();
  }, [api]);

  const colDefs = useMemo((): ColDef[] => {
    return [
      { field: "group" },
      { field: "key" },
      { field: "type" },
      { field: "value" },
    ];
  }, []);

  const getRowId = (entry: LamabdaParamEntry) => entry.key;

  return (
    <AgGridReact
      immutableData
      columnDefs={colDefs}
      rowData={entries}
      getRowNodeId={getRowId}
    />
  );
};

export default memo(Lambda);
