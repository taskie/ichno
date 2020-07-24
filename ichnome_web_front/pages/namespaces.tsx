import { NextPage } from "next";
import Head from "next/head";
import { useRouter } from "next/router";
import { uria } from "@/utils/uri";
import { defaultInstance } from "@/api/apiClient";
import { applicationName } from "@/config";
import { GetNamespacesResponse } from "@/api/types";
import Namespace from "@/components/Namespace";

type Response = GetNamespacesResponse;

type Props = { response?: Response; err?: string };

const ResponseView: React.FC<{ response: Response }> = ({ response: { namespaces } }) => {
  return (
    <>
      {namespaces.map((n) => (
        <Namespace key={n.id} namespace={n} />
      ))}
    </>
  );
};

export const NamespacePage: NextPage<Props> = (props) => {
  const router = useRouter();
  return (
    <div className="container">
      <Head>
        <title>Namespaces - {applicationName}</title>
      </Head>
      <h1>Namespaces</h1>
      {props.response != null ? <ResponseView response={props.response} /> : <p>Some error occured: {props.err}</p>}
    </div>
  );
};

NamespacePage.getInitialProps = async ({ query: rawQuery }) => {
  try {
    const path = uria`namespaces`;
    const { data } = await defaultInstance.get(path);
    return { response: data };
  } catch (err) {
    console.error(err);
    return { err: err.message };
  }
};

export default NamespacePage;
