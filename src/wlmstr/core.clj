(ns wlmstr.core
  (:require
   [babashka.fs :as fs]
   [babashka.cli :as cli]
   [babashka.process :refer [shell]]
   [clojure.edn :as edn]
   [cheshire.core :as json]
   [clojure.string :as str]
   [clojure.java.io :as io]
   [camel-snake-kebab.core :as csk]))

(defn find-papers [dir]
  (let [exts #{"avif" "jpg" "jpeg" "jxl" "png" "gif" "pnm" "pbm" "pgm" "ppm"
               "pam" "tga" "tif" "tiff" "webp" "bmp" "ff" "svg"}]
    (->> (fs/glob dir "**" {:follow-links true})
         (filter fs/regular-file?)
         (filter #(some-> (fs/extension %) str/lower-case exts))
         (map str)
         (sort))))

(defn status-file-path []
  (fs/path (or (System/getenv "XDG_DATA_HOME")
               (fs/path (System/getProperty "user.home") ".local" "share"))
           "wlmstr" "data.json"))

(defn load-status
  ([] (load-status (status-file-path)))
  ([path] (json/parse-string (slurp (fs/file path)) csk/->kebab-case-keyword)))

(defn write-status [data]
  (with-open [w (io/writer (fs/file (status-file-path)))]
    (json/generate-stream data w {:key-fn csk/->camelCaseString
                                  :pretty true})))

(defn next-paper-path [list current direction]
  (case (str/lower-case (or direction "seq"))
    ("seq" "sequence") (or (second (drop-while #(not= % current) list)) (first list))
    ("pre" "previous") (or (second (drop-while #(not= % current) (reverse list))) (last list))
    ("rnd" "random") (rand-nth list)
    (throw (ex-info (str "unknown direction: " direction) {}))))

(defn change-paper [path]
  (shell "awww img --transition-type wave --transition-duration 0.5" path))

(defn next-paper [{:keys [args]}]
  (let [status (load-status)
        papers (find-papers (:dir-path status))
        current (:paper-path status)
        direction (first args)
        next-path (next-paper-path papers current direction)]
    (change-paper next-path)
    (write-status (assoc status :paper-path next-path))
    next-path))

(defn status [_] (load-status))

(defn set-status [{:keys [opts]}]
  (let [{:keys [dir new-path]} opts]
    (write-status
     (cond-> (load-status)
       dir      (assoc :dir-path (str dir))
       new-path (assoc :paper-path (str new-path))))
    "updated"))

(defn version [_]
  (let [config (some-> (System/getProperty "babashka.config")
                       slurp
                       edn/read-string)]
    (:wlmstr/version config)))

(defn help [_]
  "usage: next [seq|pre|rnd] | set --dir DIR --new-path FILE | status | version")

(defn -main [& args]
  (println
   (cli/dispatch
    [{:cmds ["next"]    :fn next-paper}
     {:cmds ["status"]  :fn status}
     {:cmds ["set"]     :fn set-status}
     {:cmds ["version"] :fn version}
     {:cmds ["help"]    :fn help}]
    args)))
