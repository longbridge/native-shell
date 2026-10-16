---
title: Video
description: Video 组件：原生视频播放，可使用自定义控件或系统控件。
order: 10
---

# Video

视频视图用平台自带的播放器播放视频文件或视频流：iOS 和 Mac Catalyst 上是 AVPlayer，Android 上是 MediaPlayer。播放通过属性控制（`paused`、`muted`、`rate` 等），因此播放器会像其他视图一样跟随你的状态。

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

`Video` 从 [`nsh/video`](../modules/nsh-video.md) 导入，`source` 必填。另见 [Web view 与视频指南](../docs/web-view-and-video.md)。

## 属性

`Video` 接受 [view](./view.md) 的属性（不含 `children`），另外还有：

| 属性 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `source` | `string \| { uri: string }` | | 视频的 URL。 |
| `paused` | `boolean` | `false` | 暂停播放。 |
| `muted` | `boolean` | `false` | 静音。 |
| `volume` | `number` | `1` | 0..1。 |
| `rate` | `number` | `1` | 播放速度。 |
| `repeat` | `boolean` | `false` | 播放到结尾后从头再播。 |
| `controls` | `boolean` | `false` | 显示平台的播放控件。 |
| `resizeMode` | `"cover" \| "contain" \| "stretch" \| "center"` | | 视频如何适配其区域。 |
| `progressUpdateInterval` | `number` | `250` | `onProgress` 事件之间的毫秒数。 |

请通过样式给视频一个尺寸，它不会按视频自身大小调整。如果需要视频本身的比例，可以用 `onLoad` 的 `width` 和 `height` 设置 `aspectRatio`。

## 事件

| 事件 | 载荷 | 时机 |
|---|---|---|
| `onLoadStart` | 无 | 开始加载。 |
| `onLoad` | `{ duration, width, height }` | 视频已就绪：时长（秒）和尺寸。 |
| `onProgress` | `{ currentTime, duration }` | 播放期间每隔 `progressUpdateInterval` 触发（秒）。 |
| `onBuffer` | `{ isBuffering }` | 因等待数据而卡顿 / 恢复播放。 |
| `onEnd` | 无 | 播放到结尾（设置了 `repeat` 时，每轮结束都会触发）。 |
| `onError` | `{ message }` | 加载或播放失败。 |

```tsx
const ratio = signal(16 / 9);
<Video source={url} style={{ width: "100%", aspectRatio: ratio() }} onLoad={(e) => ratio.set(e.width / e.height)} />;
```

## Ref 方法

| 方法 | 说明 |
|---|---|
| `seek(seconds)` | 跳转到某个时间点。 |

## 能力声明

如果应用声明了[能力](../docs/sandbox-and-capabilities.md)，视频源必须被 `network` 允许；被拒绝的视频源不会显示任何内容。

## 平台说明

- iOS 在 `AVPlayerLayer` 中绘制视频，设置 `controls` 时使用系统播放器视图；AVFoundation 在第一次创建视频时才加载。
- Android 用 MediaPlayer 在 TextureView 上播放，`controls` 使用 MediaController。
