---
# layout: home
title: Download
editLink: false
sidebar: false
# hideExcerpt: true
---

<script setup>
  import Select from '../components/Select.vue'
  import { ref } from "vue";

  const releases = ref(undefined);

  const os = ref(undefined);

  const parseServerAssets = (assets) => {
    return {
      winX64: assets.find((d) => d.name=="ahqai-server-x86_64-pc-windows-msvc.zip"),
      winArm: assets.find((d) => d.name=="ahqai-server-aarch64-pc-windows-msvc.zip"),
      linuxX64: assets.find((d) => d.name=="ahqai-server-x86_64-unknown-linux-gnu.zip"),
      linuxArm: assets.find((d) => d.name=="ahqai-server-aarch64-unknown-linux-gnu.zip"),
      macX64: assets.find((d) => d.name=="ahqai-server-x86_64-apple-darwin.zip"),
      macArm: assets.find((d) => d.name=="ahqai-server-aarch64-apple-darwin.zip")
    }
  }

  const parseClientAssets = (assets) => {
    let output = {
      debug: {
        winX64: {
          browser_download_url: "",
          digest: ""
        },
        winArm: {
          browser_download_url: "",
          digest: ""
        },
        linuxX64: {
          rpm: { browser_download_url: "", digest: "" },
          deb: { browser_download_url: "", digest: "" }
        },
        linuxArm: {
          rpm: { browser_download_url: "", digest: "" },
          deb: { browser_download_url: "", digest: "" }
        },
        macX64: { browser_download_url: "", digest: "" },
        macArm: { browser_download_url: "", digest: "" },
        androidUniv: { browser_download_url: "", digest: "" },
        androidX64: { browser_download_url: "", digest: "" },
        androidX86: { browser_download_url: "", digest: "" },
        androidArmv7: { browser_download_url: "", digest: "" },
        androidArm64: { browser_download_url: "", digest: "" },
        androidArmMobile: { browser_download_url: "", digest: "" },
      },
      release: {
        winX64: { browser_download_url: "", digest: "" },
        winArm: { browser_download_url: "", digest: "" },
        linuxX64: {
          rpm: { browser_download_url: "", digest: "" },
          deb: { browser_download_url: "", digest: "" }
        },
        linuxArm: {
          rpm: { browser_download_url: "", digest: "" },
          deb: { browser_download_url: "", digest: "" }
        },
        macX64: { browser_download_url: "", digest: "" },
        macArm: { browser_download_url: "", digest: "" },
        androidUniv: { browser_download_url: "", digest: "" },
        androidX64: { browser_download_url: "", digest: "" },
        androidX86: { browser_download_url: "", digest: "" },
        androidArmv7: { browser_download_url: "", digest: "" },
        androidArm64: { browser_download_url: "", digest: "" },
        androidArmMobile: { browser_download_url: "", digest: "" },
      }
    };

    output.debug.winX64 = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_x64_en-US_windows-debug.msi$/.test(d.name));
    output.release.winX64 = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_x64_en-US_windows.msi$/.test(d.name));

    output.debug.winArm = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_arm64_en-US_windows-debug.msi$/.test(d.name));
    output.release.winArm = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_arm64_en-US_windows.msi$/.test(d.name));

    // LINUX
    output.release.linuxX64.deb = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_amd64_linux.deb$/.test(d.name));
    output.debug.linuxX64.deb = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_amd64_linux-debug.deb$/.test(d.name));

    output.release.linuxX64.rpm = assets.find((d) => /^AHQ.AI-(\d+.\d+.\d+-\d+).x86_64_linux.rpm$/.test(d.name));
    output.debug.linuxX64.rpm = assets.find((d) => /^AHQ.AI-(\d+.\d+.\d+-\d+).x86_64_linux-debug.rpm$/.test(d.name));

    // LINUX ARM64
    output.release.linuxArm.deb = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_arm64_linux.deb$/.test(d.name));
    output.debug.linuxArm.deb = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_arm64_linux-debug.deb$/.test(d.name));

    output.release.linuxArm.rpm = assets.find((d) => /^AHQ.AI-(\d+.\d+.\d+-\d+).aarch64_linux.rpm$/.test(d.name));
    output.debug.linuxArm.rpm = assets.find((d) => /^AHQ.AI-(\d+.\d+.\d+-\d+).aarch64_linux-debug.rpm$/.test(d.name));

    // MACOS x64
    output.release.macX64 = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_x64_darwin.dmg$/.test(d.name));
    output.debug.macX64 = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_x64_darwin-debug.dmg$/.test(d.name));

    // MACOS ARM64
    output.release.macArm = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_aarch64_darwin.dmg$/.test(d.name));
    output.debug.macArm = assets.find((d) => /^AHQ.AI_(\d+.\d+.\d+)_aarch64_darwin-debug.dmg$/.test(d.name));

    // ANDROID UNIV
    output.release.androidUniv = assets.find((d) => d.name =="app-universal-release.apk");
    output.debug.androidUniv = assets.find((d) => d.name == "app-universal-debug.apk");

    output.release.androidArmMobile = assets.find((d) => d.name =="app-arm-mobile-release.apk");
    output.debug.androidArmMobile = assets.find((d) => d.name == "app-arm-mobile-debug.apk");

    output.release.androidX64 = assets.find((d) => d.name =="app-x86_64-release.apk");
    output.debug.androidX64 = assets.find((d) => d.name == "app-x86_64-debug.apk");

    output.release.androidX86 = assets.find((d) => d.name =="app-x86-release.apk");
    output.debug.androidX86 = assets.find((d) => d.name == "app-x86-debug.apk");

    output.release.androidArmv7 = assets.find((d) => d.name =="app-arm-release.apk");
    output.debug.androidArmv7 = assets.find((d) => d.name == "app-arm-debug.apk");

    output.release.androidArm64 = assets.find((d) => d.name =="app-arm64-release.apk");
    output.debug.androidArm64 = assets.find((d) => d.name == "app-arm64-debug.apk");

    return output;
  };

  (async() => {
    const releaseData = await fetch("https://api.github.com/repos/ahqsoftorg/ahqai/releases?per_page=100", {
      cache: "force-cache"
    })
      .then((d) => d.json());

    const latestClient = await fetch("https://api.github.com/repos/ahqsoftorg/ahqai/releases/latest", {
      cache: "force-cache"
    })
      .then((d) => {
        if (!d.ok) {
          throw new Error("")
        }

        return d.json();
      })
      .catch(() => (undefined));

    const bleedingEdgeClient = releaseData.find((d) => d.tag_name.startsWith("v"));

    const latestServer = releaseData.find((d) => d.tag_name.startsWith("server-v") && !d.prerelease);

    const bleedingEdgeServer = releaseData.find((d) => d.tag_name.startsWith("server-v"));

    const outValue = {
      client: {
        latest: latestClient ? parseClientAssets(latestClient.assets) : undefined,
        bleeding: bleedingEdgeClient ? parseClientAssets(bleedingEdgeClient.assets) : undefined
      },
      server: {
        latest: latestServer ? parseServerAssets(latestServer.assets) : undefined,
        bleeding: bleedingEdgeServer ? parseServerAssets(bleedingEdgeServer.assets) : undefined
      }
    }

    releases.value = outValue;

    console.log(outValue);
  })()

  const channel = ref("latest");
  const appTypeRef = ref("release");
  const entry = ref();
  const bundle = ref();

  const x64 = ["x86", "x64", "x86_64", "Win64"];
  const arm64 = ["arm64", "aarch64", "arm"];

  async function windowsAutoFill() {
    let arch = "";

    try {
      arch = (await navigator?.userAgentData?.getHighEntropyValues(["architecture"])).architecture;

    } catch (e) {
      arch = navigator.userAgent;

      console.warn(e);
      console.log("Using fallback method");
    }

    if (arm64.some((d) => arch.toLowerCase().includes(d))) {
      entry.value = "winArm"
    } else if (x64.some((d) => arch.toLowerCase().includes(d))) {
      entry.value = "winX64"
    } else {
      entry.value = "winX64"
    }
  }

  async function macAutoFill() {
    let arch = "";

    try {
      arch = (await navigator?.userAgentData?.getHighEntropyValues(["architecture"])).architecture;

    } catch (e) {
      arch = navigator.userAgent;

      console.warn(e);
      console.log("Using fallback method");
    }

    if (arm64.some((d) => arch.toLowerCase().includes(d))) {
      entry.value = "macArm"
    } else if (x64.some((d) => arch.toLowerCase().includes(d))) {
      entry.value = "macX64"
    } else {
      entry.value = "macX64"
    }
  }

  async function androidAutoFill() {
    entry.value = "androidArmMobile"
  }

  const channelOpt = [
    { text: 'Stable', value: 'latest' },
    { text: 'Pre-Stable', value: 'bleeding' },
  ]

  const appType = [
    { text: 'Release', value: 'release' },
    { text: 'Debug', value: 'debug' },
  ];

  const winArchOptions = [
    { text: 'X64', value: 'winX64' },
    { text: 'Arm64', value: 'winArm' },
  ];

  const macArchOptions = [
    { text: 'X64', value: 'macX64' },
    { text: 'Arm64', value: 'macArm' },
  ];

  const androidArchOptions = [
    { text: 'Default', value: 'androidUniv' },
    { text: 'Mobile', value: 'androidArmMobile' },
    { text: 'Arm64', value: 'androidArm64' },
    { text: 'Armv7', value: 'androidArmv7' },
    { text: 'Intel X64', value: 'androidX64' },
    { text: 'Intel X86', value: 'androidX86' },
  ];

  const linuxArchOptions = [
    { text: 'X64', value: 'linuxX64' },
    { text: 'Arm64', value: 'linuxArm' },
  ];

  const linuxBundleOptions = [
    { text: '.deb', value: 'deb' },
    { text: '.rpm', value: 'rpm' },
  ];
</script>

<div style="margin-top:3rem;" />

# Download

AHQ AI provides two different applications:

- Server
- Client

AHQ AI hosts neither the client nor the server. We assume no legal liability for use or misuse of this application.

:::details ⚖️ Legal & Other Disclaimer

#### Disclaimer

- AHQ AI does not provide a service.
- AHQ AI is neither a product.
- AHQ AI also gives you the sourced code compiled into binaries.
- The organization is not legally bound to use or misuse of this software.
- This organization is not subject to legal jurisdiction regarding the use of this software.
- We bear no legal responsibility.

#### Other Applications

AHQ AI server utilizes Ollama for its functionality. We encourage you to read their own terms of service and privacy policy, as they govern the use of the Ollama component and are separate legal obligations from AHQ AI.

#### Open Sourced Components

AHQ AI is licensed under GPL-3.0 and utilizes many other open sourced libraries, components, frameworks for the development. Ensure that your use complies with the licenses.

#### End User Liabilities

- Ensure that the server you're connecting to is a officially built binary.
- Ensure that you accept to the terms of service of the server you are connecting to.
- Verify that the integrity of the binaries using our provided digests.

#### Optional End User Recommendation

- Ensure that the software you're using is signed by our Self Signed Certificates on the **Windows Platform**.

:::

:::details Release Details

#### Server

The server is to be hosted on your personal server (which should be a capable PC for handling LLMs)

#### Client

The client is available for a multitude of operating systems including for mobile devices like iOS and Android.

#### Types of Releases

Both are client and server has two types of releases:

- Stable: This is the version that is OFFICIALLY released and is recommended to be used.
- Latest: The absolute latest version that is OFFICIALLY built by the organization.

:::

## Client

<div v-if="!releases" class="loader" style="margin-bottom:10px;margin-left: auto;margin-right:auto;"></div>

<div v-if="releases">

:::tabs key:os
== Windows

> #### <ins>Install our Certificate (Optional; Recommended)</ins>
>
> AHQ AI Client provides a self signed certificate on Windows, which you can install in an **Elevated Powershell** by running the below command.
>
> ```powershell
> (Invoke-WebRequest -Uri "https://ahqsoftorg.github.io/ahqai/ahqai_win32.crt" -OutFile "$env:TEMP\ahqai-root-ca.crt").Headers | Out-Null; Import-Certificate -FilePath "$env:TEMP\ahqai-root-ca.crt" -CertStoreLocation Cert:\LocalMachine\Root
> ```

### Download

<span>Fill the parameters and download button will be shown, if build is available</span>

<div class="responsive-grid" style="width:100%;gap:10px;margin-bottom:5px;">
  <Select v-model="channel" :options="channelOpt" placeholder="Select Channel" />

  <Select v-model="appTypeRef" :options="appType" placeholder="Select Release" />

  <Select v-model="entry" :options="winArchOptions" placeholder="Select Architecture" />
</div>
<div style="display:flex;width:100%;margin-bottom:20px;">
  <button @click="windowsAutoFill()" class="dontknow">Autofill Dropdowns</button>
</div>
<div style="width:100%;display:flex;flex-direction:column;">
  <span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="channel && appTypeRef && entry && !releases.client?.[channel]?.[appTypeRef]?.[entry]">Unavailable</span>
<a 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.client?.[channel]?.[appTypeRef]?.[entry]!=undefined"
  :href="releases.client?.[channel]?.[appTypeRef]?.[entry]?.browser_download_url">Download</a>

<span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.client?.[channel]?.[appTypeRef]?.[entry]!=undefined">Digest:<br />{{ releases.client?.[channel]?.[appTypeRef]?.[entry]?.digest }}</span>

</div>
== MacOS

#### ⚠️ Requires Additional Steps

Mac users need to follow a few additional steps. Refer to [AHQ AI Client for MacOS](/install/mac)

#### Download

<span>Fill the parameters and download button will be shown, if build is available</span>

<div class="responsive-grid" style="width:100%;gap:10px;margin-bottom:5px;">
  <Select v-model="channel" :options="channelOpt" placeholder="Select Channel" />

  <Select v-model="appTypeRef" :options="appType" placeholder="Select Release" />

  <Select v-model="entry" :options="macArchOptions" placeholder="Select Architecture" />
</div>
<div style="display:flex;width:100%;margin-bottom:20px;">
  <button @click="macAutoFill()" class="dontknow">Autofill Dropdowns</button>
</div>
<div style="width:100%;display:flex;flex-direction:column;">
  <span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="channel && appTypeRef && entry && !releases.client?.[channel]?.[appTypeRef]?.[entry]">Unavailable</span>
<a 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.client?.[channel]?.[appTypeRef]?.[entry]!=undefined"
  :href="releases.client?.[channel]?.[appTypeRef]?.[entry]?.browser_download_url">Download</a>

<span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.client?.[channel]?.[appTypeRef]?.[entry]!=undefined">Digest:<br />{{ releases.client?.[channel]?.[appTypeRef]?.[entry]?.digest }}</span>

</div>
== Linux
<span>Fill the parameters and download button will be shown, if build is available</span>
<div class="responsive-grid" style="width:100%;gap:10px;margin-bottom:30px;">
  <Select v-model="channel" :options="channelOpt" placeholder="Select Channel" />

  <Select v-model="appTypeRef" :options="appType" placeholder="Select Release" />

  <Select v-model="entry" :options="linuxArchOptions" placeholder="Select Arch" />

  <Select v-model="bundle" :options="linuxBundleOptions" placeholder="Select Bundle" />
</div>
<div style="width:100%;display:flex;flex-direction:column;">
  <span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="channel && appTypeRef && entry && !releases.client?.[channel]?.[appTypeRef]?.[entry]">Unavailable</span>
<a 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.client?.[channel]?.[appTypeRef]?.[entry]?.[bundle]!=undefined"
  :href="releases.client?.[channel]?.[appTypeRef]?.[entry]?.[bundle]?.browser_download_url">Download</a>

<span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.client?.[channel]?.[appTypeRef]?.[entry]?.[bundle]!=undefined">Digest:<br />{{ releases.client?.[channel]?.[appTypeRef]?.[entry]?.[bundle]?.digest }}</span>

</div>
== Android
<span>Fill the parameters and download button will be shown, if build is available</span>
<div class="responsive-grid" style="width:100%;gap:10px;margin-bottom:5px;">
  <Select v-model="channel" :options="channelOpt" placeholder="Select Channel" />

  <Select v-model="appTypeRef" :options="appType" placeholder="Select Release" />

  <Select v-model="entry" :options="androidArchOptions" placeholder="Select Arch" />
</div>
<div style="display:flex;width:100%;margin-bottom:20px;">
  <button @click="androidAutoFill()" class="dontknow">Autofill Dropdowns</button>
</div>
<div style="width:100%;display:flex;flex-direction:column;">
<span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="channel && appTypeRef && entry && !releases.client?.[channel]?.[appTypeRef]?.[entry]">Unavailable</span>
  <a 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.client?.[channel]?.[appTypeRef]?.[entry]!=undefined"
  :href="releases.client?.[channel]?.[appTypeRef]?.[entry]?.browser_download_url">Download</a>

<span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.client?.[channel]?.[appTypeRef]?.[entry]!=undefined">Digest:<br />{{ releases.client?.[channel]?.[appTypeRef]?.[entry]?.digest }}</span>

</div>
== IOS

Please read the guide [here](/install/ios)

:::

</div>

::: details Supported Client OS

| OS      | Architecture              | Supported | Notes                                |
| ------- | :------------------------ | :-------: | :----------------------------------- |
| Windows | x64                       |    ✅     | Windows 10+ (Windows 11 Recommended) |
|         | arm64                     |    ✅     |                                      |
| macOS   | x64                       |    ✅     | Sideloading required                 |
|         | arm64                     |    ✅     | Sideloading required                 |
| Linux   | x64                       |    ✅     | Requires Ubuntu 22.04 or later       |
|         | arm64                     |    ✅     |                                      |
| Android | arm64                     |    ✅     |                                      |
|         | armv7                     |    ✅     |                                      |
|         | armv7, arm64              |    ✅     | Combined APK                         |
|         | x86                       |    ✅     |                                      |
|         | x86_64                    |    ✅     |                                      |
|         | x86, x86_64, armv7, arm64 |    ✅     | Combined APK                         |
| iOS     | -                         |    🟨     | Build from Scratch.                  |
|         |                           |           | Occasionally we may provide binaries |

:::

## Server

_The Server is for advanced users and requires a separate installation of Ollama, which must be configured before running the AHQ AI Server._

### Prerequisites

- Install [Ollama](https://ollama.com/download) and configure it.

### Download

<div v-if="!releases" class="loader" style="margin-bottom:10px;margin-left: auto;margin-right:auto;"></div>

<div v-if="releases">

:::tabs key:os
== Windows
<span>Fill the parameters and download button will be shown, if build is available</span>

<div class="responsive-grid" style="width:100%;gap:10px;margin-bottom:30px;">
  <Select v-model="channel" :options="channelOpt" placeholder="Select Channel" />

  <Select v-model="entry" :options="winArchOptions" placeholder="Select Arch" />
</div>
<div style="width:100%;display:flex;flex-direction:column;">
<a 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.server?.[channel]?.[entry]"
  :href="releases.server?.[channel]?.[entry]?.browser_download_url">Download</a>

<span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.server?.[channel]?.[entry]!=undefined">Digest:<br />{{ releases.server?.[channel]?.[entry]?.digest }}</span>

</div>
== MacOS
<span>Fill the parameters and download button will be shown, if build is available</span>
<div class="responsive-grid" style="width:100%;gap:10px;margin-bottom:30px;">
  <Select v-model="channel" :options="channelOpt" placeholder="Select Channel" />

  <Select v-model="entry" :options="macArchOptions" placeholder="Select Arch" />
</div>
<div style="width:100%;display:flex;flex-direction:column;">
<a 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.server?.[channel]?.[entry]"
  :href="releases.server?.[channel]?.[entry]?.browser_download_url">Download</a>

<span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.server?.[channel]?.[entry]!=undefined">Digest:<br />{{ releases.server?.[channel]?.[entry]?.digest }}</span>

</div>
== Linux
<span>Fill the parameters and download button will be shown, if build is available</span>
<div class="responsive-grid" style="width:100%;gap:10px;margin-bottom:30px;">
  <Select v-model="channel" :options="channelOpt" placeholder="Select Channel" />

  <Select v-model="entry" :options="linuxArchOptions" placeholder="Select Arch" />
</div>
<div style="width:100%;display:flex;flex-direction:column;">
  <a 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.server?.[channel]?.[entry]"
  :href="releases.server?.[channel]?.[entry]?.browser_download_url">Download</a>

<span 
  style="display:block;margin-left: auto;margin-right:auto;" 
  v-if="releases.server?.[channel]?.[entry]!=undefined">Digest:<br />{{ releases.server?.[channel]?.[entry]?.digest }}</span>

</div>
:::

</div>

::: details Supported OS for Server

| OS      | Architecture | Supported | Notes               |
| ------- | :----------- | :-------: | :------------------ |
| Windows | x64          |    ✅     | Windows 10 or above |
|         | arm64        |    ✅     |                     |
| macOS   | x64          |    ✅     |                     |
|         | arm64        |    ✅     |                     |
| Linux   | x64          |    ✅     | Ubuntu 22.04+       |
|         | arm64        |    ✅     | Ubuntu 22.04+       |

:::
