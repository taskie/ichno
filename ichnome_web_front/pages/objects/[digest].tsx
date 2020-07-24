import { NextPage } from "next";
import Head from "next/head";
import { useRouter } from "next/router";
import { uria } from "@/utils/uri";
import { defaultInstance } from "@/api/apiClient";
import { applicationName } from "@/config";
import { GetObjectResponse } from "@/api/types";
import ObjectView from "@/components/Object";
import HistoryGroup from "@/components/HistoryGroup";
import StatGroup from "@/components/StatGroup";

type Query = {
  digest: string;
};

type Response = GetObjectResponse;

type Props = { response?: Response; err?: string };

const ResponseView: React.FC<{ response: Response }> = ({ response: { object, stats, histories } }) => {
  return (
    <>
      <h2>Object</h2>
      <ObjectView object={object} />
      {stats != null ? (
        <>
          <h2>Stats</h2>
          <StatGroup stats={stats} />
        </>
      ) : undefined}
      {histories != null ? (
        <>
          <h2>Histories</h2>
          <HistoryGroup histories={histories} />
        </>
      ) : undefined}
    </>
  );
};

export const ObjectPage: NextPage<Props> = (props) => {
  const router = useRouter();
  const { query: rawQuery } = router;
  const { digest } = (rawQuery as unknown) as Query;
  return (
    <div className="container">
      <Head>
        <title>
          Object: {digest} - {applicationName}
        </title>
      </Head>
      <h1>Object: {digest.slice(0, 8)}</h1>
      {props.response != null ? <ResponseView response={props.response} /> : <p>Some error occured: {props.err}</p>}
    </div>
  );
};

ObjectPage.getInitialProps = async ({ query: rawQuery }) => {
  try {
    const { digest } = (rawQuery as unknown) as Query;
    const path = uria`objects/${digest}`;
    const { data } = await defaultInstance.get(path);
    return { response: data };
  } catch (err) {
    console.error(err);
    return { err: err.message };
  }
};

export default ObjectPage;
