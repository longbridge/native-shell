---
title: Video
description: "The Video component: native video playback with your own or the system's controls."
order: 10
---

# Video

A video view plays a video file or stream with the platform's player: AVPlayer on iOS and Mac Catalyst, MediaPlayer on
Android. Playback is controlled with props (`paused`, `muted`, `rate`, …), so the player follows your state like any
other view.

```tsx
import { signal, Text, View, type YNode } from "nsh";
import { Video } from "nsh/video";

function Player() {
  const paused = signal(true);
  const time = signal("");
  let video!: YNode;
  return (
    <View style={{ gap: 12 }}>
      <Video
        ref={video}
        source="https://example.com/clip.mp4"
        paused={paused()}
        repeat
        resizeMode="contain"
        style={{ height: 220, backgroundColor: "#000000" }}
        onProgress={(e) => time.set(`${e.currentTime.toFixed(1)} / ${e.duration.toFixed(1)} s`)}
      />
      <View style={{ flexDirection: "row", gap: 16 }}>
        <Text onPress={() => paused.set(!paused())}>{paused() ? "Play" : "Pause"}</Text>
        <Text onPress={() => video.seek(0)}>Restart</Text>
        <Text>{time()}</Text>
      </View>
    </View>
  );
}
```

`Video` comes from [`nsh/video`](../modules/nsh-video.md); `source` is required. See the
[web view and video guide](../docs/web-view-and-video.md).

## Props

`Video` takes the props of a [view](./view.md) (without `children`) plus:

| Prop | Type | Default | Description |
|---|---|---|---|
| `source` | `string \| { uri: string }` | | The video's URL. |
| `paused` | `boolean` | `false` | Playback is paused. |
| `muted` | `boolean` | `false` | No sound. |
| `volume` | `number` | `1` | 0..1. |
| `rate` | `number` | `1` | The playback speed. |
| `repeat` | `boolean` | `false` | Plays again from the start at the end. |
| `controls` | `boolean` | `false` | Shows the platform's playback controls. |
| `resizeMode` | `"cover" \| "contain" \| "stretch" \| "center"` | | How the video fits its box. |
| `progressUpdateInterval` | `number` | `250` | Milliseconds between `onProgress` events. |

Give the video a size through its style; it does not size itself to the video. Use the `width` and `height` of `onLoad`
to set an `aspectRatio` if you need the video's own proportions.

## Events

| Event | Payload | When |
|---|---|---|
| `onLoadStart` | none | Loading began. |
| `onLoad` | `{ duration, width, height }` | The video is ready: its duration in seconds and its size. |
| `onProgress` | `{ currentTime, duration }` | Every `progressUpdateInterval` while playing (seconds). |
| `onBuffer` | `{ isBuffering }` | Playback stalled for data / resumed. |
| `onEnd` | none | Playback reached the end (with `repeat`, at the end of each loop). |
| `onError` | `{ message }` | Loading or playback failed. |

```tsx
const ratio = signal(16 / 9);
<Video source={url} style={{ width: "100%", aspectRatio: ratio() }} onLoad={(e) => ratio.set(e.width / e.height)} />;
```

## Ref methods

| Method | Description |
|---|---|
| `seek(seconds)` | Jumps to a time. |

## Capabilities

If the app declares [capabilities](../docs/sandbox-and-capabilities.md), video sources must be allowed by `network`;
a refused source shows nothing.

## Platform notes

- iOS draws the video in an `AVPlayerLayer`, or in the system player view with `controls`; AVFoundation is loaded the
  first time a video is created.
- Android plays with MediaPlayer on a TextureView, with a MediaController for `controls`.
