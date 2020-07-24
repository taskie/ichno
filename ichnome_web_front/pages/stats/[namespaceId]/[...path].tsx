import { NextPage } from "next";
import Head from "next/head";
import { useRouter } from "next/router";
import { uria } from "@/utils/uri";
import { defaultInstance } from "@/api/apiClient";
import { applicationName } from "@/config";
import { GetStatResponse } from "@/api/types";
import Stat from "@/components/Stat";
import ObjectView from "@/components/Object";
import HistoryGroup from "@/components/HistoryGroup";
import StatGroup from "@/components/StatGroup";

type Query = {
  namespaceId: string;
  path: string[];
};

type Response = GetStatResponse;

type Props = { response?: Response; err?: string };

const ResponseView: React.FC<{ response: Response }> = ({ response: { stat, histories, objects, eq_stats } }) => {
  const object = objects != null ? objects["" + stat.object_id] : undefined;
  return (
    <>
      <h2>Stat</h2>
      <Stat stat={stat} />
      {histories != null ? (
        <>
          <h2>Histories</h2>
          <HistoryGroup histories={histories} />
        </>
      ) : undefined}
      {object != null ? (
        <>
          <h2>Object</h2>
          <ObjectView object={object} />
        </>
      ) : undefined}
      {eq_stats != null ? (
        <>
          <h2>Same Stats</h2>
          <StatGroup stats={eq_stats} />
        </>
      ) : undefined}
    </>
  );
};

export const StatPage: NextPage<Props> = (props) => {
  const router = useRouter();
  const { query: rawQuery } = router;
  const { namespaceId, path: statPath } = (rawQuery as unknown) as Query;
  return (
    <div className="container">
      <Head>
        <title>
          {namespaceId}/{statPath.join("/")} - {applicationName}
        </title>
      </Head>
      <h1>
        {namespaceId}/{statPath.join("/")}
      </h1>
      {props.response != null ? <ResponseView response={props.response} /> : <p>Some error occured: {props.err}</p>}
    </div>
  );
};

StatPage.getInitialProps = async ({ query: rawQuery }) => {
  try {
    const { namespaceId, path: statPath } = (rawQuery as unknown) as Query;
    const path = uria`stats/${namespaceId}/` + statPath.join("/");
    const { data } = await defaultInstance.get(path);
    return { response: data };
  } catch (err) {
    console.error(err);
    return { err: err.message };
  }
};

export default StatPage;
