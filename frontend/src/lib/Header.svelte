<script lang="ts">
    import { onMount, type Component } from "svelte";
    import WiDaySunny from "~icons/wi/day-sunny";
    import WiNightClear from "~icons/wi/night-clear";
    import WiDayCloudy from "~icons/wi/day-cloudy";
    import WiNightAltCloudy from "~icons/wi/night-alt-cloudy";
    import WiCloud from "~icons/wi/cloud";
    import WiCloudy from "~icons/wi/cloudy";
    import WiDayShowers from "~icons/wi/day-showers";
    import WiNightAltShowers from "~icons/wi/night-alt-showers";
    import WiDayRain from "~icons/wi/day-rain";
    import WiNightAltRain from "~icons/wi/night-alt-rain";
    import WiDayThunderstorm from "~icons/wi/day-thunderstorm";
    import WiNightAltThunderstorm from "~icons/wi/night-alt-thunderstorm";
    import WiDaySnow from "~icons/wi/day-snow";
    import WiNightAltSnow from "~icons/wi/night-alt-snow";
    import WiDayFog from "~icons/wi/day-fog";
    import WiNightFog from "~icons/wi/night-fog";
    import WiNa from "~icons/wi/na";

    /** OpenWeather icon codes: https://openweathermap.org/weather-conditions#Icon-list */
    export let icon: string;
    export let condition: string;
    export let conditionUntil: string = "";
    export let tempF: number;
    export let highF: number;
    export let lowF: number;
    export let date: string;
    export let time: string;

    const FAHRENHEIT_MS = 8_000;
    const CELSIUS_MS = 3_000;

    const weatherIcons: Record<string, Component> = {
        "01d": WiDaySunny,
        "01n": WiNightClear,
        "02d": WiDayCloudy,
        "02n": WiNightAltCloudy,
        "03d": WiCloud,
        "03n": WiCloud,
        "04d": WiCloudy,
        "04n": WiNightAltCloudy,
        "09d": WiDayShowers,
        "09n": WiNightAltShowers,
        "10d": WiDayRain,
        "10n": WiNightAltRain,
        "11d": WiDayThunderstorm,
        "11n": WiNightAltThunderstorm,
        "13d": WiDaySnow,
        "13n": WiNightAltSnow,
        "50d": WiDayFog,
        "50n": WiNightFog,
    };

    let unit: "F" | "C" = "F";

    const toC = (f: number) => Math.round(((f - 32) * 5) / 9);

    $: WeatherIcon = weatherIcons[icon] ?? WiNa;
    $: temp = unit === "F" ? tempF : toC(tempF);
    $: high = unit === "F" ? highF : toC(highF);
    $: low = unit === "F" ? lowF : toC(lowF);

    onMount(() => {
        let timeout: ReturnType<typeof setTimeout>;
        const schedule = () => {
            const delay = unit === "F" ? FAHRENHEIT_MS : CELSIUS_MS;
            timeout = setTimeout(() => {
                unit = unit === "F" ? "C" : "F";
                schedule();
            }, delay);
        };
        schedule();
        return () => clearTimeout(timeout);
    });
</script>

<div
    class="relative flex shrink-0 justify-between items-center bg-[#051B31] text-white font-semibold shadow-[0_3px_4px_0px_rgba(0,0,0,0.29)] overflow-hidden"
>
    <img
        class="pointer-events-none absolute inset-y-0 left-0 h-full w-auto"
        src="/header-left.png"
        alt=""
        aria-hidden="true"
    />
    <img
        class="pointer-events-none absolute inset-y-0 right-0 h-full w-auto"
        src="/header-right.png"
        alt=""
        aria-hidden="true"
    />
    <img
        class="relative z-10 h-18.5 w-18.5 my-4 ml-10"
        src="/cmu-wordmark-square-w-on-r.png"
        alt="CMU Logo"
    />
    <div
        class="absolute left-1/2 z-10 -translate-x-1/2 text-5xl whitespace-nowrap"
    >
        Live PRT Bus Arrival
    </div>
    <div class="relative z-10 flex flex-row items-center my-4.5 mr-14">
        <div class="flex flex-row items-center pr-4.5">
            <WeatherIcon class="size-14 shrink-0 text-white m-2" />
            <div class="gap-1">
                <div class="text-2xl">{condition}</div>
                {#if conditionUntil}
                    <div class="text-sm">{conditionUntil}</div>
                {/if}
            </div>
        </div>
        <div class="border-x-2 px-3 py-1.5">
            <div class="flex flex-col items-center">
                <div class="text-2xl">{temp} &deg;{unit}</div>
                <div class="flex flex-row gap-2 text-sm">
                    <div>H {high}&deg;</div>
                    <div>L {low}&deg;</div>
                </div>
            </div>
        </div>
        <div class="gap-1 pl-4.5 text-xl">
            <div>{date}</div>
            <div>{time}</div>
        </div>
    </div>
</div>
