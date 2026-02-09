import { Composition, Folder,} from "remotion";
import { CrdtExplainer, crdtExplainerSchema, CRDT_EXPLAINER_DURATION, CRDT_EXPLAINER_FPS } from "./CrdtExplainer";

// Welcome to the Remotion Three Starter Kit!
// Two compositions have been created, showing how to use
// the `ThreeCanvas` component and the `useVideoTexture` hook.

// You can play around with the example or delete everything inside the canvas.

// Remotion Docs:
// https://remotion.dev/docs

// @remotion/three Docs:
// https://remotion.dev/docs/three

// React Three Fiber Docs:
// https://docs.pmnd.rs/react-three-fiber/getting-started/introduction

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Folder name="Explainers">
        <Composition
          id="CrdtExplainer"
          component={CrdtExplainer}
          fps={CRDT_EXPLAINER_FPS}
          durationInFrames={CRDT_EXPLAINER_DURATION}
          width={1920}
          height={1080}
          schema={crdtExplainerSchema}
          defaultProps={{}}
        />
      </Folder>
    </>
  );
};
