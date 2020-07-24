import { NextPage } from "next";
import Head from "next/head";
import { useRouter } from "next/router";
import { uria } from "@/utils/uri";
import { defaultInstance } from "@/api/apiClient";
import { applicationName } from "@/config";
import { GetNamespaceResponse } from "@/api/types";
import Namespace from "@/components/Namespace";
import Stat from "@/components/Stat";
import ObjectView from "@/components/Object";
import HistoryGroup from "@/components/HistoryGroup";

type Query = {
  namespaceId: string;
};

type Response = GetNamespaceResponse;

type Props = { response?: Response; err?: string };

const ResponseView: React.FC<{ response: Response }> = ({ response: { namespace, stat, histories, objects } }) => {
  const object = stat != null && objects != null ? objects["" + stat.object_id] : undefined;
  return (
    <>
      <h2>Namespace</h2>
      <Namespace namespace={namespace} />
      <h2>Stat</h2>
      {stat != null ? <Stat stat={stat} /> : "Nothing"}
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
    </>
  );
};

export const NamespacePage: NextPage<Props> = (props) => {
  const router = useRouter();
  const { query: rawQuery } = router;
  const { namespaceId } = (rawQuery as unknown) as Query;
  return (
    <div className="container">
      <Head>
        <title>
          {namespaceId} - {applicationName}
        </title>
      </Head>
      <h1>{namespaceId}</h1>
      {props.response != null ? <ResponseView response={props.response} /> : <p>Some error occured: {props.err}</p>}
    </div>
  );
};

NamespacePage.getInitialProps = async ({ query: rawQuery }) => {
  try {
    const { namespaceId } = (rawQuery as unknown) as Query;
    const path = uria`namespaces/${namespaceId}`;
    const { data } = await defaultInstance.get(path);
    return { response: data };
  } catch (err) {
    console.error(err);
    return { err: err.message };
  }
};

export default NamespacePage;
