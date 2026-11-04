#!/usr/bin/env bash
# Makes a mod's sound effects with ElevenLabs (model eleven_text_to_sound_v3) from the prompts in
# its `sounds.tsv` (see the top of cogwork/sounds.tsv), and writes them as mono Ogg Vorbis to
# `<mod>/assets/sounds/`, their peak at -1 dBFS and the silence around them cut.
#
#   tools/sfx.sh <mod folder> [<file>...]
#
# Without files it makes every variant that has no file yet, so a run that stopped halfway
# carries on; named files (`engine/overheat`) are made again, all their variants. The raw downloads stay in `<mod>/target/sfx/`. Needs the elevenlabs CLI, ffmpeg
# and a key in ELEVENLABS_API_KEY (or ~/.config/elevenlabs/api-key).
set -euo pipefail

[[ $# -gt 0 && $1 != -h && $1 != --help ]] || { awk 'NR > 1 && !/^#/ { exit } NR > 1' "$0"; exit 0; }
folder=$1; shift
list="$folder/sounds.tsv"
[[ -f $list ]] || { echo "sfx.sh: $list does not exist" >&2; exit 2; }
for tool in elevenlabs ffmpeg jq; do
    command -v "$tool" >/dev/null || { echo "sfx.sh: $tool is not installed" >&2; exit 2; }
done
if [[ -z ${ELEVENLABS_API_KEY:-} ]]; then
    ELEVENLABS_API_KEY=$(cat ~/.config/elevenlabs/api-key)
    export ELEVENLABS_API_KEY
fi
raw="$folder/target/sfx"
out="$folder/assets/sounds"

# Whether variant $2 of sound $1 should be made in this run.
wanted() {
    if [[ ${#names[@]} -eq 0 ]]; then
        [[ ! -f $out/$1$2.ogg ]]
    else
        printf '%s\n' "${names[@]}" | grep -qxF "$1"
    fi
}

# Downloads variant $2 of sound $1 ($3 seconds, prompt influence $4, prompt $5) into $raw.
download() {
    local body
    body=$(jq -n --arg text "$5" --argjson seconds "$3" --argjson influence "$4" \
        '{text: $text, model_id: "eleven_text_to_sound_v3", duration_seconds: $seconds, prompt_influence: $influence}')
    mkdir -p "$(dirname "$raw/$1$2.wav")"
    elevenlabs text-to-sound-effects convert --json "$body" --output-format wav_44100 \
        -o "$raw/$1$2.wav" </dev/null >/dev/null
}

# Mixes wav $1 to mono with its peak at -1 dBFS, cuts the silence around it (what stays 55 dB
# below the peak) and writes it as ogg $2, which appears only once it is complete.
finish() {
    local mono="pan=mono|c0=0.5*c0+0.5*c1" cut="silenceremove=start_periods=1:start_threshold=-56dB:detection=peak"
    local peak
    peak=$(ffmpeg -hide_banner -i "$1" -af "$mono,volumedetect" -f null - </dev/null 2>&1 \
        | sed -n 's/.*max_volume: \(-\?[0-9.]*\) dB.*/\1/p')
    mkdir -p "$(dirname "$2")"
    ffmpeg -hide_banner -loglevel error -y -i "$1" \
        -af "$mono,volume=$(echo "-1 - ($peak)" | bc)dB,$cut,areverse,$cut,afade=t=in:d=0.01,areverse" \
        -c:a libvorbis -q:a 5 -f ogg "$2.part" </dev/null
    mv "$2.part" "$2"
}

names=("$@")
while IFS=$'\t' read -r file variants seconds influence prompt; do
    [[ -z $file || $file == \#* ]] && continue
    for n in $(seq "$variants"); do
        wanted "$file" "$n" || continue
        echo "sfx.sh: $file$n"
        download "$file" "$n" "$seconds" "$influence" "$prompt"
        finish "$raw/$file$n.wav" "$out/$file$n.ogg"
    done
done <"$list"
