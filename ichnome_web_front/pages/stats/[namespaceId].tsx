import { NextPage } from "next";
import Head from "next/head";
import { useRouter } from "next/router";
import { uria } from "@/utils/uri";
import { defaultInstance } from "@/api/apiClient";
import { applicationName } from "@/config";
import { GetStatsResponse } from "@/api/types";
import Namespace from "@/components/Namespace";
import Stat from "@/components/Stat";
import StatGroup from "@/components/StatGroup";

type Query = {
  namespaceId: string;
};

type Response = GetStatsResponse;

type Props = { response?: Response; err?: string };

const ResponseView: React.FC<{ response: Response }> = ({ response: { namespace, stats } }) => {
  return (
    <>
      <h2>Stats</h2>
      <StatGroup stats={stats} />
      <h2>Namespace</h2>
      <Namespace namespace={namespace} />
    </>
  );
};

export const StatsPage: NextPage<Props> = (props) => {
  const router = useRouter();
  const { query: rawQuery } = router;
  const { namespaceId } = (rawQuery as unknown) as Query;
  return (
    <div className="container">
      <Head>
        <title>
          Stats of {namespaceId} - {applicationName}
        </title>
      </Head>
      <h1>Stats of {namespaceId}</h1>
      {props.response != null ? <ResponseView response={props.response} /> : <p>Some error occured: {props.err}</p>}
    </div>
  );
};

StatsPage.getInitialProps = async ({ query: rawQuery }) => {
  try {
    const { namespaceId } = (rawQuery as unknown) as Query;
    const path = uria`stats/${namespaceId}`;
    const { data } = await defaultInstance.get(path);
    return { response: data };
  } catch (err) {
    console.error(err);
    return { err: err.message };
  }
};

export default StatsPage;
